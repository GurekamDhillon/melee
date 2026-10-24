//! Port of src/melee/it/kinds/itrshell.c: the red shell (logic table 15). Callees of unported TUs go through
//! `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itrshell.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj, a5: *mut Vec);
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::itColl_BounceOffVictim(a0) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B5E4(a0: *mut Vec, a1: *mut Vec, a2: *mut HSD_GObj) -> *mut HSD_GObj => unsafe { crate::tu::it_26B1::it_8026B5E4(a0 as _, a1 as _, a2) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026D8A4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D8A4(a0, a1) };
    fn it_8026DF34(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DF34(a0) };
    fn it_8026E32C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026E32C(a0, a1) };
    fn it_8026E414(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E414(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_8027236C(a0: *mut HSD_GObj) -> *mut HSD_GObj => unsafe { crate::tu::itcoll::it_8027236C(a0) };
    fn it_802723FC(a0: *mut HSD_GObj) -> *mut HSD_GObj => unsafe { crate::tu::itcoll::it_802723FC(a0) };
    fn it_802725D4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802725D4(a0) };
    fn it_80272674(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::it_2725::it_80272674(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272940(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272940(a0) };
    fn it_80272980(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272980(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80274C88(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80274c88(&mut *((*a0).user_data as *mut Item)) };
    fn it_80274CAC(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274CAC(a0) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_80275174(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275174(a0, a1) };
    fn it_80275414(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275414(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275444(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275444(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275474(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275474(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756E0(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_802756e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275D5C(a0: *mut HSD_GObj, a1: *mut _itECB) => unsafe { crate::tu::itmaplib::it_80275D5C(a0, a1 as _) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80276348(a0: *mut HSD_GObj, a1: *mut Vec) -> i32 => unsafe { crate::tu::itmaplib::it_80276348(a0, a1 as _) };
    fn it_80276CB8(a0: *mut HSD_GObj) => unsafe { crate::tu::itmaplib::it_80276CB8(a0) };
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

/// The TU-local `itRShell_Attrs` (the article's special attributes; all 4-byte members, so the native and the
/// GameCube layouts agree).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct itRShell_Attrs {
    pub x0: f32,
    pub x4: f32,
    pub x8: f32,
    pub xC: f32,
    pub x10: f32,
    pub x14: Vec3,
    pub x20: f32,
    pub x24: f32,
    pub x28: f32,
    pub x2C: f32,
    pub pad30: [u8; 8],
    pub x38: f32,
    pub x3C: f32,
    pub x40: f32,
    pub x44: f32,
    pub x48: Vec3,
    pub x54: i32,
}

/// `ItemStateTable it_803F5C48[]`
pub static mut it_803F5C48: [ItemStateTable; 8] = [
    st_row(-1, pred_b(itRshell_UnkMotion0_Anim), Some(itRshell_UnkMotion0_Phys), pred_b(itRshell_UnkMotion0_Coll)),
    st_row(-1, pred_b(itRshell_UnkMotion1_Anim), Some(itRshell_UnkMotion1_Phys), pred_b(itRshell_UnkMotion1_Coll)),
    st_row(-1, pred_b(itRshell_UnkMotion2_Anim), Some(itRshell_UnkMotion2_Phys), None),
    st_row(0, pred_b(itRshell_UnkMotion3_Anim), Some(itRshell_UnkMotion3_Phys), pred_b(itRshell_UnkMotion3_Coll)),
    st_row(0, pred_b(itRshell_UnkMotion4_Anim), Some(itRshell_UnkMotion4_Phys), pred_b(itRshell_UnkMotion4_Coll)),
    st_row(0, pred_b(itRshell_UnkMotion5_Anim), Some(itRshell_UnkMotion5_Phys), pred_b(itRshell_UnkMotion5_Coll)),
    st_row(0, pred_b(itRshell_UnkMotion6_Anim), Some(itRshell_UnkMotion6_Phys), pred_b(itRshell_UnkMotion6_Coll)),
    st_row(-1, pred_b(itRshell_UnkMotion7_Anim), Some(itRshell_UnkMotion7_Phys), pred_b(itRshell_UnkMotion7_Coll)),
];

/// `Item_UpdateRollingShellRotation` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_UpdateRollingShellRotation(gobj: *mut Item_GObj, ip: *mut Item, jobj: *mut HSD_JObj, rotation_rate: *const f32) {
    ext::it_80276CB8(gobj);
    let jobj = HSD_JObjGetChild(jobj);
    let vx = (*ip).x40_vel.x;
    HSD_JObjAddRotationY(jobj, *rotation_rate * (if vx < 0.0 { -vx } else { vx }));
}

pub unsafe extern "C" fn it_8028CFE0(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itRShell_Attrs;
    let mut v: Vec3;
    /// @todo Shared code with #it_8028B8D8.
    if (*ip).xDD4_itemVar.rshell.xDDC <= 0.0 {
        let jobj = GET_JOBJ(gobj);
        v = (*attrs).x48;
        let temp = -(*ip).facing_dir;
        v.x *= temp;
        ext::efAsync_Spawn(gobj, &mut (*GET_ITEM(gobj)).xBC0 as *mut _ as *mut c_void, 2, 1029, jobj, &mut v);
        (*ip).xDD4_itemVar.rshell.xDDC = (*attrs).x44;
    } else {
        (*ip).xDD4_itemVar.rshell.xDDC -= 1.0;
    }
}

pub unsafe extern "C" fn it_8028D090(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itRShell_Attrs;
    ext::it_80275D5C(gobj, &mut (*ip).xC0C);
    it_8028DAE4(gobj);
    (*ip).x40_vel.x = it_8028D56C(gobj, (*attrs).x4, (*ip).facing_dir);
    ext::it_80272980(gobj);
}

pub unsafe extern "C" fn it_8028D100(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itRShell_Attrs;
    let kind = (*ip).msid;
    let mut did_hit: i32 = 0;

    // 0, 1, 3, 4, 7: set the velocity; 5, 6: add to it; anything else (2, negative, > 7): only check the speed.
    match kind {
        0 | 1 | 3 | 4 | 7 => {
            did_hit = 1;
            (*ip).x40_vel.x = -(*ip).xCCC_incDamageDirection * (((*ip).xCA0 as f32) * (*attrs).x20);
        }
        5 | 6 => {
            it_8028D390(gobj);
            (*ip).x40_vel.x = fmadds(-(*ip).xCCC_incDamageDirection, ((*ip).xCA0 as f32) * (*attrs).x24, (*ip).x40_vel.x);
        }
        _ => {}
    }

    let mut vel = (*ip).x40_vel.x;
    if vel < 0.0 {
        vel = -vel;
    }
    if vel > (*attrs).x10 {
        ext::it_8027236C(gobj);
        if did_hit != 0 {
            ext::it_802756D0(gobj);
        }
        if (*ip).ground_or_air == GA_Air {
            it_8028E170(gobj);
            return;
        }
        it_8028DAE4(gobj);
        return;
    }
    ext::it_802725D4(gobj);
}

pub unsafe extern "C" fn it_8028D26C(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itRShell_Attrs;
    let kind = (*ip).msid;
    let mut did_hit: i32 = 0;

    match kind {
        0 | 1 | 3 | 4 | 7 => {
            did_hit = 1;
            (*ip).x40_vel.x = (*attrs).x28 * -(*ip).xCD0;
        }
        5 | 6 => {
            it_8028D390(gobj);
            (*ip).x40_vel.x = fmadds((*attrs).x2C, -(*ip).xCD0, (*ip).x40_vel.x);
        }
        _ => {}
    }

    let mut vel = (*ip).x40_vel.x;
    if vel < 0.0 {
        vel = -vel;
    }
    if vel > (*attrs).x10 {
        ext::it_802723FC(gobj);
        if did_hit != 0 {
            ext::it_802756D0(gobj);
        }
        if (*ip).ground_or_air == GA_Air {
            it_8028E170(gobj);
            return;
        }
        it_8028DAE4(gobj);
        return;
    }
    ext::it_802725D4(gobj);
}

pub unsafe extern "C" fn it_8028D390(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.rshell.xDEC = (*ip).xDD4_itemVar.rshell.xDEC.wrapping_sub(1);
    if (*ip).xDD4_itemVar.rshell.xDEC == 0 {
        (*ip).xDD4_itemVar.rshell.xDF0 = 1;
    }
}

pub unsafe extern "C" fn it_8028D3B8(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);

    (*ip).xDD4_itemVar.rshell.xDEC = (*ip).xDD4_itemVar.rshell.xDEC.wrapping_sub(1);
    if (*ip).xDD4_itemVar.rshell.xDEC == 0 {
        (*ip).xDD4_itemVar.rshell.xDF0 = 1;
    }
    if (*ip).xDD4_itemVar.rshell.xDF0 == 0 {
        ext::it_802756E0(gobj);
        ext::it_80275444(gobj);
        (*ip).x40_vel.x = it_8028D56C(gobj, (*ip).x40_vel.x, (*ip).xCB8_outDamageDirection);
        ext::it_80272980(gobj);
        if (*ip).xDD4_itemVar.rshell.xDF4_b0() != 0 {
            let mut rand_max: u8 = (*ext::it_804D6D28).x48_byte;
            rand_max &= 0xF;
            (*ip).xDD4_itemVar.rshell.xDE8 = ext::HSD_Randi(rand_max as i32);
        } else {
            (*ip).xDD4_itemVar.rshell.set_xDF4_b0(1);
        }
        if (*ip).msid != 6 && (*ip).msid != 5 {
            if (*ip).ground_or_air == GA_Air {
                it_8028E170(gobj);
                return;
            }
            it_8028DAE4(gobj);
        }
    }
}

pub unsafe extern "C" fn fn_8028D4A8(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.x = -(*ip).x40_vel.x;
    (*ip).xDD4_itemVar.rshell.xDE0 = -(*ip).xDD4_itemVar.rshell.xDE0;
    ext::it_80272980(gobj);
}

pub unsafe extern "C" fn it_8028D4E4(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let vel = (*ip).x40_vel.x;
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itRShell_Attrs;

    let abs_vel = if vel < 0.0 { -vel } else { vel };

    if abs_vel > (*attrs).xC {
        let sign: i32 = if vel < 0.0 { -1 } else { 1 };
        (*ip).x40_vel.x = sign as f32;
        (*ip).x40_vel.x *= (*attrs).xC;
    }
}

/// `itRshell_ClampVel` (static inline)
#[inline]
unsafe fn itRshell_ClampVel(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let vel = (*ip).x40_vel.x;
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itRShell_Attrs;

    let abs_vel = if vel < 0.0 { -vel } else { vel };

    if abs_vel > (*attrs).xC {
        let sign: i32 = if vel < 0.0 { -1 } else { 1 };
        (*ip).x40_vel.x = sign as f32;
        (*ip).x40_vel.x *= (*attrs).xC;
    }
}

pub unsafe extern "C" fn it_8028D56C(gobj: *mut Item_GObj, f1: f32, f2: f32) -> f32 {
    let attrs = (*(*GET_ITEM(gobj)).xC4_article_data).x4_specialAttributes as *mut itRshellAttributes;
    // (f1 * -x14.x) + (-x14.y * f2): the left product is contracted with the add
    let f = fmadds(f1, -(*attrs).x14.x, -(*attrs).x14.y * f2);
    f * (*attrs).x14.z
}

pub unsafe extern "C" fn it_3F14_Logic15_Spawned(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itRShell_Attrs;
    (*ip).xDD4_itemVar.rshell.xDD4 = (*attrs).x0;
    ext::it_80275174(gobj, (*ip).xDD4_itemVar.rshell.xDD4);
    (*ip).xDD4_itemVar.rshell.xDEC = (*attrs).x54;
    (*ip).xDD4_itemVar.rshell.xDF0 = 0;
    (*ip).xDD4_itemVar.rshell.xDD8 = (*attrs).x3C;
    (*ip).xDD4_itemVar.rshell.xDE8 = 1;
    (*ip).xDD4_itemVar.rshell.set_xDF4_b0(0);
    (*ip).xDD4_itemVar.rshell.xDE0 = 0.0;
    it_8028D7F0(gobj);
}

pub unsafe extern "C" fn it_8028D62C(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itRShell_Attrs;
    ext::it_8026B390(gobj);
    ext::it_80275414(gobj);
    ext::it_802756E0(gobj);
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    (*ip).xDD4_itemVar.rshell.xDD8 = (*attrs).x3C;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    ext::it_80274CAC(gobj);
    (*ip).jumped_on = pred_b(it_8028E6C0);
}

pub unsafe extern "C" fn itRshell_UnkMotion0_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itRshell_UnkMotion0_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itRshell_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(it_8028D7F0));
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itRShell_Attrs;
    if (*ip).ground_or_air == GA_Ground {
        Item_UpdateRollingShellRotation(gobj, ip, jobj, &(*attrs).x38);
    }
    false
}

pub unsafe extern "C" fn it_8028D7F0(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itRshell_UnkMotion1_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itRshell_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itRshell_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.rshell.xDE8 != 0 {
        ext::it_8026E32C(gobj, Some(it_8028D62C));
    } else {
        return ext::it_8026DF34(gobj) != 0;
    }
    false
}

pub unsafe extern "C" fn itRShell_Logic15_PickedUp(gobj: *mut Item_GObj) {
    ext::it_80275474(gobj);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itRshell_UnkMotion2_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itRshell_UnkMotion2_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn it_3F14_Logic15_Thrown(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itRShell_Attrs;
    (*ip).xDD4_itemVar.rshell.xDE8 = 1;
    (*ip).xDD4_itemVar.rshell.xDD8 = (*attrs).x3C;
    ext::Item_80268E5C(gobj, 3, 6);
    ext::Item_8026AE84(ip, 0xF2, 0x7F, 0x40);
}

pub unsafe extern "C" fn itRshell_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.rshell.xDD8 <= 0.0 {
        if (*ip).xDCD_flag.anon_0.b5() == 0 {
            ext::it_80275444(gobj);
        }
    } else {
        (*ip).xDD4_itemVar.rshell.xDD8 -= 1.0;
    }
    false
}

pub unsafe extern "C" fn itRshell_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itRshell_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E414(gobj, Some(it_8028D090));
    false
}

pub unsafe extern "C" fn it_3F14_Logic15_Dropped(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itRShell_Attrs;
    (*ip).xDD4_itemVar.rshell.xDD8 = (*attrs).x3C;
    ext::Item_80268E5C(gobj, 4, 6);
}

pub unsafe extern "C" fn itRshell_UnkMotion4_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.rshell.xDD8 <= 0.0 {
        if (*ip).xDCD_flag.anon_0.b5() == 0 {
            ext::it_80275444(gobj);
        }
    } else {
        (*ip).xDD4_itemVar.rshell.xDD8 -= 1.0;
    }
    false
}

pub unsafe extern "C" fn itRshell_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itRshell_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E414(gobj, Some(it_8028D090));
    false
}

/// `itRshell_StopInit` (static inline)
#[inline]
unsafe fn itRshell_StopInit(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itRShell_Attrs;
    ext::it_8026B3A8(gobj);
    ext::it_80274C88(gobj);
    ext::it_80275158(gobj, (*ext::it_804D6D28).x30_lifetime as _);
    (*ip).xD5C = 0;
    (*ip).xDD4_itemVar.rshell.xDD8 = (*attrs).x3C;
    (*ip).xDD4_itemVar.rshell.xDE4 = 0.0;
    (*ip).xDD4_itemVar.rshell.xDDC = (*attrs).x44;
    ext::it_80275414(gobj);
    itRshell_ClampVel(gobj);
    ext::it_80272980(gobj);
}

pub unsafe extern "C" fn it_8028DAE4(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.y = 0.0;
    itRshell_StopInit(gobj);
    if (*ip).msid == 0 || (*ip).msid == 1 || (*ip).msid == 7 {
        ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
    } else {
        ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE | ITEM_HIT_PRESERVE);
    }
}

pub unsafe extern "C" fn itRshell_UnkMotion5_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.rshell.xDD4 <= 0.0 {
        return true;
    }
    (*ip).xDD4_itemVar.rshell.xDD4 -= 1.0;
    if (*ip).xDD4_itemVar.rshell.xDD8 <= 0.0 {
        if (*ip).xDCD_flag.anon_0.b5() == 0 {
            ext::it_80275444(gobj);
        }
    } else {
        (*ip).xDD4_itemVar.rshell.xDD8 -= 1.0;
    }
    if (*ip).msid == 5 {
        it_8028CFE0(gobj);
    }
    false
}

/// `itRshell_UM5_Accel` (static inline)
#[inline]
unsafe fn itRshell_UM5_Accel(gobj: *mut Item_GObj, target: *mut Vec3) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itRShell_Attrs;
    let sign: i32 = if (*target).x - (*ip).pos.x < 0.0 { -1 } else { 1 };
    (*ip).xDD4_itemVar.rshell.xDE0 = (*attrs).x8 * (sign as f32);
}

/// `itRshell_UM5_AddVelAndCheck` (static inline)
#[inline]
unsafe fn itRshell_UM5_AddVelAndCheck(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.x += (*ip).xDD4_itemVar.rshell.xDE0;
    let sign_xDE0: i32 = if (*ip).xDD4_itemVar.rshell.xDE0 < 0.0 { -1 } else { 1 };
    let sign_vx: i32 = if (*ip).x40_vel.x < 0.0 { -1 } else { 1 };
    sign_vx == sign_xDE0
}

/// `itRshell_UM5_MaybeBrake` (static inline)
#[inline]
unsafe fn itRshell_UM5_MaybeBrake(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itRShell_Attrs;
    if (*ip).x5D4_hitboxes[0].hit.state == HitCapsule_Disabled {
        let vx = (*ip).x40_vel.x;
        if (if vx < 0.0 { -vx } else { vx }) <= (*attrs).x10 {
            ext::it_80272674(gobj, 0);
        }
    }
}

pub unsafe extern "C" fn itRshell_UnkMotion5_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itRShell_Attrs;
    let mut target_pos: Vec3 = core::mem::zeroed();

    if (*ip).xDD4_itemVar.rshell.xDE4 <= 0.0 {
        if !ext::it_8026B5E4(&mut (*ip).pos, &mut target_pos, null_mut()).is_null() {
            itRshell_UM5_Accel(gobj, &mut target_pos);
            if itRshell_UM5_AddVelAndCheck(gobj) {
                it_8028D4E4(gobj);
            }
            itRshell_UM5_MaybeBrake(gobj);
            ext::it_80272980(gobj);
        }
        (*ip).xDD4_itemVar.rshell.xDE4 = (*attrs).x40;
    } else {
        (*ip).x40_vel.x += (*ip).xDD4_itemVar.rshell.xDE0;
        let sign_xDE0: i32 = if (*ip).xDD4_itemVar.rshell.xDE0 < 0.0 { -1 } else { 1 };
        let sign_vx: i32 = if (*ip).x40_vel.x < 0.0 { -1 } else { 1 };
        if sign_vx == sign_xDE0 {
            itRshell_ClampVel(gobj);
        }
        ext::it_80272980(gobj);
        (*ip).xDD4_itemVar.rshell.xDE4 -= 1.0;
    }
}

/// `itRshell_UM5C_Reverse` (static inline)
#[inline]
unsafe fn itRshell_UM5C_Reverse(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.x = -(*ip).x40_vel.x;
    (*ip).xDD4_itemVar.rshell.xDE0 = -(*ip).xDD4_itemVar.rshell.xDE0;
}

/// `itRshell_UM5C_GroundSpin` (static inline)
#[inline]
unsafe fn itRshell_UM5C_GroundSpin(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itRShell_Attrs;
    let mut jobj = GET_JOBJ(gobj);

    if (*ip).ground_or_air == GA_Ground {
        ext::it_80276CB8(gobj);
        jobj = HSD_JObjGetChild(jobj);
        let vx = (*ip).x40_vel.x;
        HSD_JObjAddRotationY(jobj, (*attrs).x38 * (if vx < 0.0 { -vx } else { vx }));
    }
}

pub unsafe extern "C" fn itRshell_UnkMotion5_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let mut v: Vec3 = core::mem::zeroed();

    if ext::it_8026D8A4(gobj, Some(fn_8028D4A8)) == 0 {
        ext::it_802762BC(ip);
        it_8028E170(gobj);
    } else {
        if ext::it_80276348(gobj, &mut v) != 0 {
            itRshell_UM5C_Reverse(gobj);
            ext::it_80272980(gobj);
            ext::it_80275444(gobj);
        }
        itRshell_UM5C_GroundSpin(gobj);
    }
    false
}

pub unsafe extern "C" fn it_8028E170(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    itRshell_StopInit(gobj);
    if (*ip).msid == 0 || (*ip).msid == 1 || (*ip).msid == 7 {
        ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE);
    } else {
        ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE | ITEM_HIT_PRESERVE);
    }
}

pub unsafe extern "C" fn itRshell_UnkMotion6_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = (*gobj).user_data as *mut Item;
    if !((*ip).xDD4_itemVar.rshell.xDD4 <= 0.0) {
        (*ip).xDD4_itemVar.rshell.xDD4 -= 1.0;
        if (*ip).xDD4_itemVar.rshell.xDD8 <= 0.0 {
            if (*ip).xDCD_flag.anon_0.b5() == 0 {
                ext::it_80275444(gobj);
            }
        } else {
            (*ip).xDD4_itemVar.rshell.xDD8 -= 1.0;
        }
        if (*ip).msid == 5 {
            it_8028CFE0(gobj);
        }
    }
    false
}

pub unsafe extern "C" fn itRshell_UnkMotion6_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itRshell_UnkMotion6_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E414(gobj, Some(it_8028D090));
    false
}

pub unsafe extern "C" fn it_3F14_Logic15_EnteredAir(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).xDC8_word.flags.set_x1F(1);
    ext::Item_80268E5C(gobj, 7, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itRshell_UnkMotion7_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itRshell_UnkMotion7_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itRshell_UnkMotion7_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E8C4(gobj, Some(it_8028D62C), Some(it_8028D7F0));
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itRShell_Attrs;
    if (*ip).ground_or_air == GA_Ground {
        Item_UpdateRollingShellRotation(gobj, ip, jobj, &(*attrs).x38);
    }
    false
}

pub unsafe extern "C" fn it_3F14_Logic15_DmgDealt(gobj: *mut Item_GObj) -> bool {
    let ip = (*gobj).user_data as *mut Item;
    ext::it_80272940(gobj);
    it_8028D3B8(gobj);
    (*ip).xDD4_itemVar.rshell.xDF0 != 0
}

pub unsafe extern "C" fn it_3F14_Logic15_DmgReceived(gobj: *mut Item_GObj) -> bool {
    let ip = (*gobj).user_data as *mut Item;
    ext::it_80272940(gobj);
    it_8028D100(gobj);
    (*ip).xDD4_itemVar.rshell.xDF0 != 0
}

pub unsafe extern "C" fn itRShell_Logic15_Reflected(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    ext::it_80273030(gobj);
    (*ip).xDD4_itemVar.rshell.xDD4 = (*ip).xD48_halfLifeTimer;
    false
}

pub unsafe extern "C" fn itRShell_Logic15_Clanked(gobj: *mut Item_GObj) -> bool {
    ext::itColl_BounceOffVictim(gobj);
    false
}

pub unsafe extern "C" fn it_3F14_Logic15_HitShield(gobj: *mut Item_GObj) -> bool {
    let ip = (*gobj).user_data as *mut Item;
    if (*ip).msid == 3 || (*ip).msid == 4 {
        ext::itColl_BounceOffVictim(gobj);
    } else if (*ip).msid == 5 || (*ip).msid == 6 {
        it_8028D3B8(gobj);
        return (*ip).xDD4_itemVar.rshell.xDF0 != 0;
    }
    false
}

pub unsafe extern "C" fn itRShell_Logic15_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 3 || (*ip).msid == 4 {
        ext::itColl_BounceOffShield(gobj);
    }
    false
}

pub unsafe extern "C" fn it_8028E6C0(gobj: *mut Item_GObj) -> bool {
    let ip = (*gobj).user_data as *mut Item;
    ext::it_80272940(gobj);
    it_8028D26C(gobj);
    ext::Item_8026AE84(ip, 0xF1, 0x7F, 0x40);
    (*ip).xDD4_itemVar.rshell.xDF0 != 0
}

pub unsafe extern "C" fn itRShell_Logic15_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
