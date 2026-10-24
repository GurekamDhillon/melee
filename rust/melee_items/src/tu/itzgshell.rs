//! Port of src/melee/it/kinds/itzgshell.c: the green Koopa shell of the Subspace Emissary (logic table 11; its red
//! sibling `itzrshell.c` shares every state function through `it_803F87F0`). `zgshell.xE08_b0` = "stunned timer
//! running", `xE08_b1` = "restart the attack hitbox", `xE1C_b0` = "recovering from a bounce"; `xE0C` is a general
//! counter and `vel` the previous bone position. Callees of unported TUs go through `ext` (unset: panics
//! "TODO"). Checked against the C (whole TU, shipping contraction) by tests/kinds_itzgshell.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::cabi::{f2i32, srem};
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
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_Randf() -> f32;
    fn HSD_Randi(a0: i32) -> i32;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8);
    fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj, a5: *mut Vec);
    fn grZakoGenerator_801CACB8(a0: *mut HSD_GObj);
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32;
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj);
    fn it_8026B390(a0: *mut HSD_GObj);
    fn it_8026B3A8(a0: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_8026DB40(a0: *mut HSD_GObj) -> i32;
    fn it_8026E32C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32;
    fn it_8026E414(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_80271534(a0: *mut HSD_GObj, a1: i32, a2: *mut HurtCapsule);
    fn it_80271590(a0: *mut HSD_GObj, a1: i32, a2: *mut HurtCapsule);
    fn it_8027236C(a0: *mut HSD_GObj) -> *mut HSD_GObj;
    fn it_802723FC(a0: *mut HSD_GObj) -> *mut HSD_GObj;
    fn it_802725D4(a0: *mut HSD_GObj);
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32);
    fn it_80272940(a0: *mut HSD_GObj);
    fn it_80272980(a0: *mut HSD_GObj);
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32;
    fn it_80273030(a0: *mut HSD_GObj) -> i32;
    fn it_8027429C(a0: *mut HSD_GObj, a1: *mut Vec);
    fn it_80274C88(a0: *mut HSD_GObj);
    fn it_80274CAC(a0: *mut HSD_GObj);
    fn it_80275158(a0: *mut HSD_GObj, a1: f32);
    fn it_80275210(a0: *mut HSD_GObj);
    fn it_80275414(a0: *mut HSD_GObj);
    fn it_80275444(a0: *mut HSD_GObj);
    fn it_80275474(a0: *mut HSD_GObj);
    fn it_802754D4(a0: *mut HSD_GObj);
    fn it_802756D0(a0: *mut HSD_GObj);
    fn it_802756E0(a0: *mut HSD_GObj);
    fn it_8027570C(a0: *mut HSD_GObj, a1: i32);
    fn it_8027572C(a0: *mut HSD_GObj, a1: i32);
    fn it_80275D5C(a0: *mut HSD_GObj, a1: *mut _itECB);
    fn it_802762B0(a0: *mut Item);
    fn it_802762BC(a0: *mut Item);
    fn it_80276CB8(a0: *mut HSD_GObj);
    fn it_80277040(a0: *mut HSD_GObj) -> i32;
    fn it_8027770C(a0: *mut HSD_GObj) -> i32;
    fn it_8027B5B0(a0: i32, a1: *mut Vec, a2: *mut HSD_JObj, a3: *mut Vec, a4: i32) -> *mut HSD_GObj;
    fn it_8027B730(a0: *mut HSD_GObj);
    fn it_8027C56C(a0: *mut HSD_GObj, a1: f32);
    fn it_8027C794(a0: *mut HSD_GObj) -> i32;
    fn it_802DD7F0(a0: i32, a1: *mut Vec, a2: *mut Vec, a3: i32) -> *mut HSD_GObj;
    fn it_802DDA84(a0: *mut HSD_GObj) -> i32;
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec);
    fn mpCollSetFacingDir(a0: *mut CollData, a1: i32);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D28: *mut ItemCommonData = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

/// `HSD_JObjGetRotationY` (baselib/jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjGetRotationY(jobj: *mut HSD_JObj) -> f32 {
    hsd_assert(!jobj.is_null(), "(jobj)");
    (*jobj).rotate.y
}

/// `typedef struct itGShell_Attrs` (the `.c` file's own record of `Article.x4_specialAttributes`;
/// `itZGShell_Attrs` is the same block seen as `{ pad[0x38]; x38; Vec x3C }`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct itGShell_Attrs {
    pub x0: f32,
    pub x4: f32,
    pub x8: f32,
    pub xC: f32,
    pub x10: f32,
    pub x14: f32,
    pub x18: f32,
    pub x1C: f32,
    pub x20: f32,
    pub x24: f32,
    pub x28: f32,
    pub x2C: f32,
    pub x30: f32,
    pub x34: f32,
    pub x38: f32,
    pub x3C: Vec3,
}
melee_core::gc_layout!(itGShell_Attrs, "itGShell_Attrs", size 0x48, align 4, [
    x0 @ 0x0: 4 "f32",
    x4 @ 0x4: 4 "f32",
    x8 @ 0x8: 4 "f32",
    xC @ 0xC: 4 "f32",
    x10 @ 0x10: 4 "f32",
    x14 @ 0x14: 4 "f32",
    x18 @ 0x18: 4 "f32",
    x1C @ 0x1C: 4 "f32",
    x20 @ 0x20: 4 "f32",
    x24 @ 0x24: 4 "f32",
    x28 @ 0x28: 4 "f32",
    x2C @ 0x2C: 4 "f32",
    x30 @ 0x30: 4 "f32",
    x34 @ 0x34: 4 "f32",
    x38 @ 0x38: 4 "f32",
    x3C @ 0x3C: 12 "Vec3",
]);

#[inline]
fn ABS(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

#[inline]
unsafe fn attrs_of(ip: *mut Item) -> *mut itGShell_Attrs {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itGShell_Attrs
}

/// `ItemStateTable it_803F86C8[]`
pub static mut it_803F86C8: [ItemStateTable; 12] = [
    st_row(0, pred_b(itZrshell_UnkMotion0_Anim), Some(itZrshell_UnkMotion0_Phys), pred_b(itZrshell_UnkMotion0_Coll)),
    st_row(0, pred_b(itZrshell_UnkMotion1_Anim), Some(itZrshell_UnkMotion1_Phys), pred_b(itZrshell_UnkMotion1_Coll)),
    st_row(0, pred_b(itZrshell_UnkMotion2_Anim), Some(itZrshell_UnkMotion2_Phys), None),
    st_row(1, pred_b(itZrshell_UnkMotion3_Anim), Some(itZrshell_UnkMotion3_Phys), pred_b(itZrshell_UnkMotion3_Coll)),
    st_row(1, pred_b(itZrshell_UnkMotion4_Anim), Some(itZrshell_UnkMotion4_Phys), pred_b(itZrshell_UnkMotion4_Coll)),
    st_row(1, pred_b(itZrshell_UnkMotion6_Anim), Some(itZrshell_UnkMotion6_Phys), pred_b(itZrshell_UnkMotion6_Coll)),
    st_row(0, pred_b(itZrshell_UnkMotion6_Anim), Some(itZrshell_UnkMotion6_Phys), pred_b(itZrshell_UnkMotion6_Coll)),
    st_row(1, pred_b(itZrshell_UnkMotion8_Anim), Some(itZrshell_UnkMotion8_Phys), pred_b(itZrshell_UnkMotion8_Coll)),
    st_row(0, pred_b(itZrshell_UnkMotion8_Anim), Some(itZrshell_UnkMotion8_Phys), pred_b(itZrshell_UnkMotion8_Coll)),
    st_row(0, pred_b(itZrshell_UnkMotion9_Anim), Some(itZrshell_UnkMotion9_Phys), pred_b(itZrshell_UnkMotion9_Coll)),
    st_row(2, pred_b(itZrshell_UnkMotion10_Anim), Some(itZrshell_UnkMotion10_Phys), pred_b(itZrshell_UnkMotion10_Coll)),
    st_row(3, pred_b(itZrshell_UnkMotion11_Anim), Some(itZrshell_UnkMotion11_Phys), pred_b(itZrshell_UnkMotion11_Coll)),
];

/// `Item_UpdateRollingShellRotation` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_UpdateRollingShellRotation(gobj: *mut Item_GObj, ip: *mut Item, jobj: *mut HSD_JObj, rotation_rate: *const f32) {
    ext::it_80276CB8(gobj);
    let jobj = HSD_JObjGetChild(jobj);
    HSD_JObjAddRotationY(jobj, *rotation_rate * ABS((*ip).x40_vel.x));
}

/// `itUpdateVelocityFromBone` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn itUpdateVelocityFromBone(jobj: *mut HSD_JObj, ip: *mut Item, previous_pos: *mut Vec3) {
    let mut pos: Vec3 = core::mem::zeroed();
    let mut zero: Vec3 = core::mem::zeroed();
    if !jobj.is_null() {
        zero.z = 0.0;
        zero.y = 0.0;
        zero.x = 0.0;
        HSD_JObjGetTranslation(jobj, &mut pos);
        (*ip).x40_vel.x = (*ip).facing_dir * (pos.z - (*previous_pos).z);
        (*ip).x40_vel.y = pos.y - (*previous_pos).y;
        (*ip).x40_vel.z = pos.x - (*previous_pos).x;
        *previous_pos = pos;
        HSD_JObjSetTranslate(jobj, &mut zero);
    }
}

/// 2DDB38: spawns the trail effect every `x38` frames.
pub unsafe extern "C" fn it_802DDB38(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    let mut v: Vec3 = core::mem::zeroed();
    if (*ip).xDD4_itemVar.zgshell.xDF8 <= 0.0 {
        let jobj = GET_JOBJ(gobj);
        v = (*attrs).x3C;
        v.x *= -(*ip).facing_dir;
        ext::efAsync_Spawn(gobj, &raw mut (*GET_ITEM(gobj)).xBC0 as *mut c_void, 2, 1029, jobj, &mut v);
        (*ip).xDD4_itemVar.zgshell.xDF8 = (*attrs).x38;
    } else {
        (*ip).xDD4_itemVar.zgshell.xDF8 -= 1.0;
    }
}

/// 2DDBE8: counts the stun timer down.
pub unsafe extern "C" fn it_802DDBE8(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    if (*ip).xDD4_itemVar.zgshell.xE08_b0() != 0 {
        (*ip).xDD4_itemVar.zgshell.xDFC -= 1.0;
        if (*ip).xDD4_itemVar.zgshell.xDFC <= 0.0 {
            (*ip).xDD4_itemVar.zgshell.set_xE08_b0(0);
            (*ip).xDD4_itemVar.zgshell.xDFC = (*attrs).x28;
            ext::it_8027572C(gobj, 0);
            if (*ip).xDD4_itemVar.zgshell.xE08_b1() != 0 {
                ext::it_802756E0(gobj);
            }
        }
    }
}

pub unsafe extern "C" fn fn_802DDC8C(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    ext::it_80275D5C(gobj, &mut (*ip).xC0C);
    if ABS((*ip).x40_vel.x) < (*attrs).x8 {
        (*ip).x40_vel.z = 0.0;
        (*ip).x40_vel.y = 0.0;
        (*ip).x40_vel.x = 0.0;
        if ext::it_80277040(gobj) == 0 {
            it_802DE0F0(gobj);
        } else {
            it_2725_Logic11_EnteredAir(gobj);
        }
    } else {
        it_802DE6F0(gobj);
    }
}

/// 2DDD38: damage received: knockback by state, then bounce / break.
pub unsafe extern "C" fn it_802DDD38(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);

    match (*ip).msid {
        0 | 1 | 3 | 4 | 9 | 10 => {
            (*ip).x40_vel.x = -(*ip).xCCC_incDamageDirection * ((*ip).xCA0 as f32 * (*attrs).x14);
            (*ip).xDD4_itemVar.zgshell.set_xE08_b1(1);
        }
        5 | 6 | 7 | 8 => {
            (*ip).x40_vel.x = fmadds(-(*ip).xCCC_incDamageDirection, (*ip).xCA0 as f32 * (*attrs).x14, (*ip).x40_vel.x);
            (*ip).xDD4_itemVar.zgshell.set_xE08_b1(1);
        }
        _ => {}
    }

    if (*ip).msid == 0xB {
        ext::Camera_RequestQuake(QuakeKind_Small as u32, &mut (*ip).pos);
        it_802DF9F8(gobj);
    } else {
        ext::it_8027236C(gobj);
        ext::it_802756D0(gobj);
        if (*ip).ground_or_air == GA_Air {
            it_802DEC80(gobj);
        } else {
            it_802DE6F0(gobj);
        }
        (*ip).xDD4_itemVar.zgshell.set_xE08_b0(1);
        ext::it_8027570C(gobj, 0);
    }
}

/// `get_attrs` (permuterslop static inline)
#[inline]
unsafe fn get_attrs(arg0: *mut Item) -> *mut itGShell_Attrs {
    attrs_of(arg0)
}

/// 2DDEB4: jumped on / hit shield: bounce away.
pub unsafe extern "C" fn it_802DDEB4(gobj: *mut Item_GObj) {
    let ip = (*gobj).user_data as *mut Item;
    let attrs = get_attrs(ip);

    match (*ip).msid {
        5 | 6 | 7 | 8 | 10 => {
            let ip = (*gobj).user_data as *mut Item;
            let attrs = get_attrs(ip);
            ext::it_802756D0(gobj);
            ext::it_80275444(gobj);
            (*ip).x40_vel.x = -(*ip).x40_vel.x * (*attrs).xC * ext::HSD_Randf();
            (*ip).x40_vel.y = (*attrs).x10;
            ext::it_802762BC(ip);
            (*ip).xDD4_itemVar.zgshell.set_xE1C_b0(1);
            (*ip).xDD4_itemVar.zgshell.xE04 = (*attrs).x34;
            it_802DE320(gobj);
            (*ip).xDD4_itemVar.zgshell.set_xE08_b1(1);
        }
        0 | 1 | 3 | 4 | 9 => {
            let ip = (*gobj).user_data as *mut Item;
            (*ip).x40_vel.x = (*attrs).x1C * -(*ip).xCD0;
            ext::it_802723FC(gobj);
            (*ip).xDD4_itemVar.zgshell.set_xE08_b1(1);
            if (*ip).ground_or_air == GA_Air {
                it_802DEC80(gobj);
            } else {
                it_802DE6F0(gobj);
            }
        }
        _ => {}
    }

    if (*ip).msid == 0xB {
        ext::Camera_RequestQuake(QuakeKind_Small as u32, &mut (*ip).pos);
        it_802DF9F8(gobj);
    } else {
        ext::it_802756D0(gobj);
        (*ip).xDD4_itemVar.zgshell.set_xE08_b0(1);
        ext::it_8027570C(gobj, 0);
    }
}

/// 2DE040: spawn setup.
pub unsafe extern "C" fn it_802DE040(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    ext::it_8027B730(gobj);
    ext::it_80275210(gobj);
    (*ip).xDD4_itemVar.zgshell.set_xE08_b0(0);
    (*ip).xDD4_itemVar.zgshell.xDFC = (*attrs).x28;
    (*ip).xDD4_itemVar.zgshell.set_xE08_b1(1);
    (*ip).xDD4_itemVar.zgshell.xE10 = 0;
    (*ip).xDD4_itemVar.zgshell.xE14 = 0;
    (*ip).xDD4_itemVar.zgshell.xE0C = 0;
    (*ip).xDD4_itemVar.zgshell.set_xE1C_b0(0);
    (*ip).xDD4_itemVar.zgshell.xE04 = (*attrs).x34;
    ext::it_80271590(gobj, 0, &raw mut (*ip).xDD4_itemVar.zgshell.x58_hurtInit as *mut HurtCapsule);
    it_802DE320(gobj);
}

/// 2DE0F0: rest on the ground.
pub unsafe extern "C" fn it_802DE0F0(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_8026B390(gobj);
    ext::it_80275444(gobj);
    ext::it_802754D4(gobj);
    ext::it_802756E0(gobj);
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    (*ip).xDD4_itemVar.zgshell.xE0C = 0;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
    (*ip).xDD4_itemVar.zgshell.set_xE1C_b0(0);
    ext::it_80274CAC(gobj);
    (*ip).jumped_on = pred_b(fn_802DFE7C);
}

pub unsafe extern "C" fn itZrshell_UnkMotion0_Anim(arg0: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itZrshell_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let counter = (*ip).xDD4_itemVar.zgshell.xE0C;
    let attrs = attrs_of(ip);
    if counter as f32 > (*attrs).x2C {
        it_802DF230(gobj);
    } else {
        (*ip).xDD4_itemVar.zgshell.xE0C = counter.wrapping_add(1);
    }
}

pub unsafe extern "C" fn itZrshell_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(it_802DE320));
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let attrs = attrs_of(ip);
    if (*ip).ground_or_air == GA_Ground {
        Item_UpdateRollingShellRotation(gobj, ip, jobj, &(*attrs).x20);
    }
    false
}

pub unsafe extern "C" fn it_802DE320(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
    ext::it_80274C88(gobj);
}

pub unsafe extern "C" fn itZrshell_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = (*gobj).user_data as *mut Item;
    if (*ip).xDD4_itemVar.zgshell.xE1C_b0() != 0 && (*ip).xDD0_flag.anon_0.b0() != 1 {
        if (*ip).xDD4_itemVar.zgshell.xE04 <= 0.0 {
            ext::it_80274CAC(gobj);
            (*ip).jumped_on = pred_b(fn_802DFE7C);
        } else {
            (*ip).xDD4_itemVar.zgshell.xE04 -= 1.0;
        }
    }
    false
}

pub unsafe extern "C" fn itZrshell_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itZrshell_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E32C(gobj, Some(it_802DE0F0));
    false
}

pub unsafe extern "C" fn it_2725_Logic11_PickedUp(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_80275474(gobj);
    (*ip).xDD4_itemVar.zgshell.xE0C = 0;
    (*ip).xDD4_itemVar.zgshell.set_xE1C_b0(0);
    ext::Item_80268E5C(gobj, 2, 2);
}

pub unsafe extern "C" fn itZrshell_UnkMotion2_Anim(arg0: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itZrshell_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let counter = (*ip).xDD4_itemVar.zgshell.xE0C;
    let attrs = attrs_of(ip);
    if counter as f32 > (*attrs).x2C {
        it_802DF230(gobj);
    } else {
        (*ip).xDD4_itemVar.zgshell.xE0C = counter.wrapping_add(1);
    }
}

pub unsafe extern "C" fn itZGShell_Logic11_Thrown(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 3, 6);
    ext::Item_8026AE84(ip, 0xF2, 0x7F, 0x40);
}

pub unsafe extern "C" fn itZrshell_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = (*gobj).user_data as *mut Item;
    let attrs = attrs_of(ip);
    if (*ip).xDD4_itemVar.zgshell.xE08_b0() != 0 {
        (*ip).xDD4_itemVar.zgshell.xDFC -= 1.0;
        if (*ip).xDD4_itemVar.zgshell.xDFC <= 0.0 {
            (*ip).xDD4_itemVar.zgshell.set_xE08_b0(0);
            (*ip).xDD4_itemVar.zgshell.xDFC = (*attrs).x28;
            ext::it_8027572C(gobj, 0);
            if (*ip).xDD4_itemVar.zgshell.xE08_b1() != 0 {
                ext::it_802756E0(gobj);
            }
        }
    }
    false
}

pub unsafe extern "C" fn itZrshell_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itZrshell_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E414(gobj, Some(fn_802DDC8C));
    false
}

pub unsafe extern "C" fn itZGShell_Logic11_Dropped(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 4, 6);
}

pub unsafe extern "C" fn itZrshell_UnkMotion4_Anim(gobj: *mut Item_GObj) -> bool {
    it_802DDBE8(gobj);
    false
}

pub unsafe extern "C" fn itZrshell_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itZrshell_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E414(gobj, Some(fn_802DDC8C));
    false
}

/// 2DE6F0: starts sliding on the ground.
pub unsafe extern "C" fn it_802DE6F0(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);

    ext::it_8026B3A8(gobj);
    ext::it_80275474(gobj);
    ext::it_80275414(gobj);
    ext::it_80275158(gobj, (*ext::it_804D6D28).x30_lifetime as f32);
    (*ip).xD5C = 0;
    (*ip).xDD4_itemVar.zgshell.xDF4 = (*attrs).x24;
    (*ip).xDD4_itemVar.zgshell.xDF8 = (*attrs).x38;
    (*ip).xDD4_itemVar.zgshell.set_xE1C_b0(0);
    if ABS((*ip).x40_vel.x) > (*attrs).x4 {
        (*ip).x40_vel.x = if (*ip).x40_vel.x < 0.0 { -1.0 } else { 1.0 };
        (*ip).x40_vel.x *= (*attrs).x4;
    }
    ext::it_80272980(gobj);
    (*ip).xDD4_itemVar.zgshell.xE0C = 0;
    it_802DFFA0(gobj);
    if ABS((*ip).x40_vel.x) < (*attrs).x8 {
        ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE as u32);
        ext::it_8026B390(gobj);
        ext::it_802754D4(gobj);
    } else {
        ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE as u32);
    }
    ext::it_80274CAC(gobj);
    (*ip).jumped_on = pred_b(fn_802DFE7C);
}

/// `it_802DDB38_inline` (static inline)
#[inline]
unsafe fn it_802DDB38_inline(gobj: *mut Item_GObj, v: *mut Vec3) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    if (*ip).xDD4_itemVar.zgshell.xDF8 <= 0.0 {
        let jobj = GET_JOBJ(gobj);
        *v = (*attrs).x3C;
        (*v).x *= -(*ip).facing_dir;
        ext::efAsync_Spawn(gobj, &raw mut (*GET_ITEM(gobj)).xBC0 as *mut c_void, 2, 1029, jobj, v);
        (*ip).xDD4_itemVar.zgshell.xDF8 = (*attrs).x38;
    } else {
        (*ip).xDD4_itemVar.zgshell.xDF8 -= 1.0;
    }
}

pub unsafe extern "C" fn itZrshell_UnkMotion6_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = (*gobj).user_data as *mut Item;
    let mut v: Vec3 = core::mem::zeroed();
    if (*ip).xDD4_itemVar.zgshell.xDF4 <= 0.0 {
        if (*ip).xDCD_flag.anon_0.b5() == 0 {
            ext::it_80275444(gobj);
        }
    } else {
        (*ip).xDD4_itemVar.zgshell.xDF4 -= 1.0;
    }
    it_802DDBE8(gobj);
    if (*ip).msid == 6 || (*ip).msid == 5 {
        it_802DDB38_inline(gobj, &mut v);
    }
    false
}

pub unsafe extern "C" fn itZrshell_UnkMotion6_Phys(gobj: *mut Item_GObj) {
    let mut ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);

    if ABS((*ip).x40_vel.x) < (*attrs).x8 {
        if (*ip).xDC8_word.flags.x15() == 0 {
            ext::it_8026B390(gobj);
            ext::it_802725D4(gobj);
            ext::it_802754D4(gobj);
        }
    }

    if ABS((*ip).x40_vel.x) < (*attrs).x30 {
        (*ip).x40_vel.z = 0.0;
        (*ip).x40_vel.y = 0.0;
        (*ip).x40_vel.x = 0.0;
        if ext::it_80277040(gobj) == 0 {
            ip = GET_ITEM(gobj);
            ext::it_8026B390(gobj);
            ext::it_80275444(gobj);
            ext::it_802756E0(gobj);
            (*ip).x40_vel.z = 0.0;
            (*ip).x40_vel.y = 0.0;
            (*ip).x40_vel.x = 0.0;
            (*ip).xDD4_itemVar.zgshell.xE0C = 0;
            ext::Item_80268E5C(gobj, 0, ITEM_UNK_0x1 as u32);
            (*ip).xDD4_itemVar.zgshell.set_xE1C_b0(0);
            ext::it_80274CAC(gobj);
            (*ip).jumped_on = pred_b(fn_802DFE7C);
        } else {
            it_2725_Logic11_EnteredAir(gobj);
        }
    }
}

pub unsafe extern "C" fn itZrshell_UnkMotion6_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(it_802DEC80));
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let attrs = attrs_of(ip);
    if (*ip).ground_or_air == GA_Ground {
        Item_UpdateRollingShellRotation(gobj, ip, jobj, &(*attrs).x20);
    }
    if ext::it_8027770C(gobj) != 0 {
        ext::it_80272980(gobj);
    }
    false
}

/// 2DEC80: slides in the air.
pub unsafe extern "C" fn it_802DEC80(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);

    ext::it_8026B3A8(gobj);
    ext::it_80275474(gobj);
    ext::it_80275414(gobj);
    ext::it_80275158(gobj, (*ext::it_804D6D28).x30_lifetime as f32);
    (*ip).xD5C = 0;
    (*ip).xDD4_itemVar.zgshell.xDF4 = (*attrs).x24;
    (*ip).xDD4_itemVar.zgshell.xDF8 = (*attrs).x38;
    (*ip).xDD4_itemVar.zgshell.set_xE1C_b0(0);
    if ABS((*ip).x40_vel.x) > (*attrs).x4 {
        (*ip).x40_vel.x = if (*ip).x40_vel.x < 0.0 { -1.0 } else { 1.0 };
        (*ip).x40_vel.x *= (*attrs).x4;
    }
    ext::it_80272980(gobj);
    (*ip).xDD4_itemVar.zgshell.xE0C = 0;
    if ABS((*ip).x40_vel.x) < (*attrs).x8 {
        ext::Item_80268E5C(gobj, 8, ITEM_ANIM_UPDATE as u32);
        ext::it_8026B390(gobj);
    } else {
        ext::Item_80268E5C(gobj, 7, ITEM_ANIM_UPDATE as u32);
    }
    ext::it_80274CAC(gobj);
    (*ip).jumped_on = pred_b(fn_802DFE7C);
}

pub unsafe extern "C" fn itZrshell_UnkMotion8_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = (*gobj).user_data as *mut Item;
    if (*ip).xDD4_itemVar.zgshell.xDF4 <= 0.0 {
        if (*ip).xDCD_flag.anon_0.b5() == 0 {
            ext::it_80275444(gobj);
        }
    } else {
        (*ip).xDD4_itemVar.zgshell.xDF4 -= 1.0;
    }
    it_802DDBE8(gobj);
    if (*ip).msid == 6 || (*ip).msid == 5 {
        it_802DDB38(gobj);
    }
    false
}

pub unsafe extern "C" fn itZrshell_UnkMotion8_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itZrshell_UnkMotion8_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E414(gobj, Some(fn_802DDC8C));
    false
}

pub unsafe extern "C" fn it_2725_Logic11_EnteredAir(gobj: *mut Item_GObj) {
    let ip = (*gobj).user_data as *mut Item;
    if (*ip).msid == 0xB || (*ip).msid == 0xA {
        return;
    }
    it_802DFFA0(gobj);
    (*ip).xDD4_itemVar.zgshell.set_xE1C_b0(0);
    (*ip).xDC8_word.flags.set_x1F(1);
    ext::Item_80268E5C(gobj, 9, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itZrshell_UnkMotion9_Anim(arg0: *mut Item_GObj) -> bool {
    false
}

/// `itZGShell_StopAndIdle` (static inline)
#[inline]
unsafe fn itZGShell_StopAndIdle(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_8026B390(gobj);
    ext::it_80275444(gobj);
    ext::it_802756E0(gobj);
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    (*ip).xDD4_itemVar.zgshell.xE0C = 0;
    ext::Item_80268E5C(gobj, 0, ITEM_UNK_0x1 as u32);
    (*ip).xDD4_itemVar.zgshell.set_xE1C_b0(0);
    ext::it_80274CAC(gobj);
    (*ip).jumped_on = pred_b(fn_802DFE7C);
}

pub unsafe extern "C" fn itZrshell_UnkMotion9_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);

    if ABS((*ip).x40_vel.x) < (*attrs).x30 {
        (*ip).x40_vel.z = 0.0;
        (*ip).x40_vel.y = 0.0;
        (*ip).x40_vel.x = 0.0;
        itZGShell_StopAndIdle(gobj);
    }
    if (*ip).pos.x == (*ip).xDD4_itemVar.zgshell.vel.x && (*ip).pos.y == (*ip).xDD4_itemVar.zgshell.vel.y {
        (*ip).x40_vel.z = 0.0;
        (*ip).x40_vel.y = 0.0;
        (*ip).x40_vel.x = 0.0;
        itZGShell_StopAndIdle(gobj);
    }
    (*ip).xDD4_itemVar.zgshell.vel.x = (*ip).pos.x;
    (*ip).xDD4_itemVar.zgshell.vel.y = (*ip).pos.y;
}

pub unsafe extern "C" fn itZrshell_UnkMotion9_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E8C4(gobj, Some(it_802DE0F0), Some(it_802DE320));
    let ip = GET_ITEM(gobj);
    let mut jobj = GET_JOBJ(gobj);
    let attrs = attrs_of(ip);
    if (*ip).ground_or_air == GA_Ground {
        ext::it_80276CB8(gobj);
        jobj = HSD_JObjGetChild(jobj);
        HSD_JObjAddRotationY(jobj, (*attrs).x20 * ABS((*ip).x40_vel.x));
    }
    false
}

/// 2DF230: shell breaks open into the Koopa (state 0xB), restoring the rotation first.
pub unsafe extern "C" fn it_802DF230(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);

    (*ip).xC9C = 0;
    (*ip).xCA0 = 0;
    ext::it_802756E0(gobj);
    ext::it_8026B3A8(gobj);
    (*ip).xDC8_word.flags.set_x1F(0);
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    it_802DFF14(gobj, 1);
    (*ip).xDD4_itemVar.zgshell.xE0C = 0x14;

    let jobj = GET_JOBJ(gobj);
    // MTXRadToDeg(a) = a * 57.29577951f, MTXDegToRad(a) = a * 0.01745329252f
    let mut angle: f32 = HSD_JObjGetRotationY(jobj) * 57.29577951;
    let child = HSD_JObjGetChild(jobj);
    // `angle += MTXRadToDeg(..)`: the product is fused into the compound add
    angle = fmadds(HSD_JObjGetRotationY(child), 57.29577951, angle);

    if -90.0 == angle || 90.0 == angle {
        (*ip).facing_dir = if angle < 0.0 { -1.0 } else { 1.0 };
        (*ip).xDD4_itemVar.zgshell.xE00 = 0.0;
        HSD_JObjSetRotationY(jobj, angle * 0.01745329252);
        HSD_JObjSetRotationY(child, 0.0);
    } else {
        let factor: f32;
        if angle > 2147483600.0 || angle < -2147483600.0 {
            // HSD_ASSERTREPORT(0x3A1, 0, "*** ZGShell Restoration Rot Y Irregul!\n")
            panic!("*** ZGShell Restoration Rot Y Irregul!");
        }
        angle = srem(f2i32(angle as f64), 360) as f32;
        HSD_JObjSetRotationY(jobj, angle * 0.01745329252);
        HSD_JObjSetRotationY(child, 0.0);
        if 0.0 == angle {
            (*ip).facing_dir = if ext::HSD_Randi(2) != 0 { -1.0 } else { 1.0 };
            factor = 90.0 * (*ip).facing_dir;
        } else if angle < 180.0 {
            (*ip).facing_dir = 1.0;
            factor = 90.0 - angle;
        } else {
            (*ip).facing_dir = -1.0;
            factor = 270.0 - angle;
        }
        (*ip).xDD4_itemVar.zgshell.xE00 = factor / (*ip).xDD4_itemVar.zgshell.xE0C as f32;
    }

    if (*ip).xDC8_word.flags.x13() != 0 {
        ext::it_8027429C(gobj, &mut (*ip).x40_vel);
        (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x14_fall_speed_max;
        ext::it_8026B3A8(gobj);
        (*ip).xDD4_itemVar.zgshell.xE18 = 1;
        ext::it_802762BC(ip);
    } else {
        (*ip).xDD4_itemVar.zgshell.xE18 = 0;
    }
    (*ip).xDD4_itemVar.zgshell.xE14 = 0x23;
    it_802DFFA0(gobj);
    ext::Item_80268E5C(gobj, 0xB, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itZrshell_UnkMotion11_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let mut pos: Vec3 = core::mem::zeroed();

    if (*ip).xDD4_itemVar.zgshell.xE0C != 0 {
        let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        HSD_JObjAddRotationY(jobj, 0.017453292 * (*ip).xDD4_itemVar.zgshell.xE00);
        (*ip).xDD4_itemVar.zgshell.xE0C = (*ip).xDD4_itemVar.zgshell.xE0C.wrapping_sub(1);
    }
    if ext::it_80272C6C(gobj) != 0 {
        let saved_vy = (*ip).x40_vel.y;
        it_802DFFB8((*(*ip).xBBC_dynamicBoneTable).bones[1], ip);
        if (*ip).xDD4_itemVar.zgshell.xE14 != 0 {
            (*ip).xDD4_itemVar.zgshell.xE14 = (*ip).xDD4_itemVar.zgshell.xE14.wrapping_sub(1);
        } else if (*ip).ground_or_air == GA_Air {
            if (*ip).x40_vel.y != 0.0 {
                (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
                if (*ip).x40_vel.y > saved_vy {
                    (*ip).x40_vel.y = saved_vy - (*(*ip).xCC_item_attr).x10_fall_speed;
                }
            } else {
                (*ip).x40_vel.y = (*(*ip).xCC_item_attr).x14_fall_speed_max;
            }
        } else {
            itResetVelocity(ip);
            ext::it_80276CB8(gobj);
        }
    } else if (*ip).ground_or_air == GA_Ground {
        ext::lb_8000B1CC(GET_JOBJ(gobj), null_mut(), &mut pos);
        it_802DFF14(gobj, 0);
        ext::it_802756D0(gobj);
        let spawn = ext::it_802DD7F0((*ip).xDD4_itemVar.zgshell.xE10, &mut pos, null_mut(), f2i32((*ip).facing_dir as f64));
        if !spawn.is_null() {
            (*GET_ITEM(spawn)).xDD4_itemVar.zgshell.xDD8 = (*ip).xDD4_itemVar.zgshell.xDD8;
            (*ip).xDD4_itemVar.zgshell.xDD8 = f32::from_bits(-1i32 as u32);
        }
        return true;
    }
    false
}

pub unsafe extern "C" fn itZrshell_UnkMotion11_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).x40_vel.y > 0.0 {
        ext::it_802762BC(ip);
    }
}

pub unsafe extern "C" fn itZrshell_UnkMotion11_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = (*gobj).user_data as *mut Item;
    if (*ip).ground_or_air == GA_Air {
        if ext::it_8026DB40(gobj) & 1 != 0 {
            ext::it_802762B0(ip);
            (*ip).x40_vel.z = 0.0;
            (*ip).x40_vel.y = 0.0;
            (*ip).x40_vel.x = 0.0;
        }
    } else {
        ext::it_8026D62C(gobj, Some(it_802DE320));
    }
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802DF9F8(gobj: *mut Item_GObj) {
    let ip = (*gobj).user_data as *mut Item;
    ext::it_802762BC(ip);
    (*ip).xDC8_word.flags.set_x1F(0);
    it_802DFF14(gobj, 0);
    ext::it_802756D0(gobj);
    ext::Item_80268E5C(gobj, 0xA, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itZrshell_UnkMotion10_Anim(gobj: *mut Item_GObj) -> bool {
    let mut ip = GET_ITEM(gobj);
    if ext::it_80272C6C(gobj) == 0 && (*ip).ground_or_air == GA_Ground {
        if ext::it_80277040(gobj) == 0 {
            ip = GET_ITEM(gobj);
            ext::it_8026B390(gobj);
            ext::it_80275444(gobj);
            ext::it_802754D4(gobj);
            ext::it_802756E0(gobj);
            (*ip).x40_vel.z = 0.0;
            (*ip).x40_vel.y = 0.0;
            (*ip).x40_vel.x = 0.0;
            (*ip).xDD4_itemVar.zgshell.xE0C = 0;
            ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
            (*ip).xDD4_itemVar.zgshell.set_xE1C_b0(0);
            ext::it_80274CAC(gobj);
            (*ip).jumped_on = pred_b(fn_802DFE7C);
        } else {
            ip = GET_ITEM(gobj);
            if (*ip).msid != 0xB && (*ip).msid != 0xA {
                it_802DFFA0(gobj);
                (*ip).xDD4_itemVar.zgshell.set_xE1C_b0(0);
                (*ip).xDC8_word.flags.set_x1F(1);
                ext::Item_80268E5C(gobj, 9, ITEM_ANIM_UPDATE as u32);
            }
        }
    }
    false
}

pub unsafe extern "C" fn itZrshell_UnkMotion10_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
}

pub unsafe extern "C" fn itZrshell_UnkMotion10_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = (*gobj).user_data as *mut Item;
    if (*ip).ground_or_air == GA_Air {
        if ext::it_8026DB40(gobj) & 1 != 0 {
            ext::it_802762B0(ip);
            (*ip).x40_vel.z = 0.0;
            (*ip).x40_vel.y = 0.0;
            (*ip).x40_vel.x = 0.0;
        }
    } else {
        ext::it_8026D62C(gobj, Some(it_802DE320));
    }
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn itZGShell_Logic11_DmgDealt(gobj: *mut Item_GObj) -> bool {
    ext::it_80272940(gobj);
    false
}

pub unsafe extern "C" fn itZGShell_Logic11_DmgReceived(gobj: *mut Item_GObj) -> bool {
    ext::it_80272940(gobj);
    it_802DDD38(gobj);
    false
}

pub unsafe extern "C" fn itZGShell_Logic11_Reflected(gobj: *mut Item_GObj) -> bool {
    ext::it_80273030(gobj) != 0
}

pub unsafe extern "C" fn it_2725_Logic11_Clanked(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    ext::it_802756D0(gobj);
    ext::it_80275444(gobj);
    (*ip).x40_vel.x = -(*ip).x40_vel.x * (*attrs).xC * ext::HSD_Randf();
    (*ip).x40_vel.y = (*attrs).x10;
    ext::it_802762BC(ip);
    (*ip).xDD4_itemVar.zgshell.set_xE1C_b0(1);
    (*ip).xDD4_itemVar.zgshell.xE04 = (*attrs).x34;
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
    ext::it_80274C88(gobj);
    false
}

pub unsafe extern "C" fn it_2725_Logic11_HitShield(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 3 || (*ip).msid == 4 {
        ext::itColl_BounceOffVictim(gobj);
    } else if ((*ip).msid as u32).wrapping_sub(5) <= 3 {
        it_2725_Logic11_Clanked(gobj);
    }
    false
}

pub unsafe extern "C" fn itZGShell_Logic11_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 3 || (*ip).msid == 4 {
        ext::itColl_BounceOffShield(gobj);
    }
    false
}

pub unsafe extern "C" fn fn_802DFE7C(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    ext::it_80272940(gobj);
    it_802DDEB4(gobj);
    ext::Item_8026AE84(ip, 0xF1, 0x7F, 0x40);
    false
}

pub unsafe extern "C" fn it_802DFED4(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe extern "C" fn itZGShell_Logic11_Destroyed(gobj: *mut Item_GObj) {
    ext::grZakoGenerator_801CACB8(gobj);
}

/// 2DFF14: sets the hurtbox from the stored init data (`arg1`: the doubled, raised one for the broken shell).
pub unsafe extern "C" fn it_802DFF14(gobj: *mut Item_GObj, arg1: i32) {
    let ip = GET_ITEM(gobj);
    let mut capsule: itGShell_HurtInit = (*ip).xDD4_itemVar.zgshell.x58_hurtInit;
    if arg1 != 0 {
        capsule.a_offset.y = 6.0;
        capsule.scale *= 2.0;
    }
    ext::it_80271534(gobj, 0, &mut capsule as *mut itGShell_HurtInit as *mut HurtCapsule);
}

/// 2DFFA0 (static)
pub unsafe extern "C" fn it_802DFFA0(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.zgshell.vel.x = 0.0;
    (*ip).xDD4_itemVar.zgshell.vel.y = 0.0;
    (*ip).xDD4_itemVar.zgshell.vel.z = 0.0;
}

pub unsafe extern "C" fn it_802DFFB8(jobj: *mut HSD_JObj, ip: *mut Item) {
    itUpdateVelocityFromBone(jobj, ip, &raw mut (*ip).xDD4_itemVar.zgshell.vel);
}

/// `const Vec3 it_803B86E8`
pub static it_803B86E8: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

/// 2E0100: spawns a green shell.
pub unsafe extern "C" fn it_802E0100(arg0: i32, pos: *mut Vec3, facing_int: i32) -> *mut Item_GObj {
    let mut vel: Vec3 = it_803B86E8;
    let spawn_gobj = ext::it_8027B5B0(It_Kind_ZGShell, pos, null_mut(), &mut vel, 1);
    if !spawn_gobj.is_null() {
        let mut spawn_ip = GET_ITEM(spawn_gobj);
        (*spawn_ip).facing_dir = facing_int as f32;
        ext::it_8027C56C(spawn_gobj, (*spawn_ip).facing_dir);
        ext::mpCollSetFacingDir(&mut (*spawn_ip).x378_itemColl, if (*spawn_ip).facing_dir == -1.0 { -1 } else { 1 });
        (*spawn_ip).xDD4_itemVar.zgshell.xE10 = arg0;

        if ext::it_802DDA84(spawn_gobj) != 0 {
            ext::it_802762B0(spawn_ip);
            if ext::it_80277040(spawn_gobj) == 0 {
                spawn_ip = GET_ITEM(spawn_gobj);
                ext::it_8026B390(spawn_gobj);
                ext::it_80275444(spawn_gobj);
                ext::it_802754D4(spawn_gobj);
                ext::it_802756E0(spawn_gobj);
                (*spawn_ip).x40_vel.z = 0.0;
                (*spawn_ip).x40_vel.y = 0.0;
                (*spawn_ip).x40_vel.x = 0.0;
                (*spawn_ip).xDD4_itemVar.zgshell.xE0C = 0;
                ext::Item_80268E5C(spawn_gobj, 0, ITEM_ANIM_UPDATE as u32);
                (*spawn_ip).xDD4_itemVar.zgshell.set_xE1C_b0(0);
                ext::it_80274CAC(spawn_gobj);
                (*spawn_ip).jumped_on = pred_b(fn_802DFE7C);
            } else {
                spawn_ip = GET_ITEM(spawn_gobj);
                if (*spawn_ip).msid != 0xB && (*spawn_ip).msid != 0xA {
                    (*spawn_ip).xDD4_itemVar.zgshell.vel.x = 0.0;
                    (*spawn_ip).xDD4_itemVar.zgshell.vel.y = 0.0;
                    (*spawn_ip).xDD4_itemVar.zgshell.vel.z = 0.0;
                    (*spawn_ip).xDD4_itemVar.zgshell.set_xE1C_b0(0);
                    (*spawn_ip).xDC8_word.flags.set_x1F(1);
                    ext::Item_80268E5C(spawn_gobj, 9, ITEM_ANIM_UPDATE as u32);
                }
            }
        } else {
            ext::it_802762BC(spawn_ip);
            ext::Item_80268E5C(spawn_gobj, 1, ITEM_ANIM_UPDATE as u32);
            ext::it_80274C88(spawn_gobj);
        }
    }
    spawn_gobj
}
