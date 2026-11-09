//! Native port of src/melee/it/kinds/itgshell.c: the green shell item (spawn, sliding and bouncing
//! states, damage / shield / reflect logic callbacks). Callees in other TUs go through `ext`.
//! Checked against the C (the whole TU, shipping contraction, item/jobj state hashed field by
//! field) by tests/tu_itgshell.rs.
//!
//! One fused site: `x40_vel.x += -xCCC_incDamageDirection * (xCA0 * x14)` in `it_8028BAD8` (fmadds).
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
use std::ptr::null_mut as null_mut_itkinds;
#[allow(unused_imports)]
use crate::itkinds::{itGShell_Attrs as itGShell_Attrs_itkinds, sa};
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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj, a5: *mut Vec);
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::itColl_BounceOffVictim(a0) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026B390(a0) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026B3A8(a0) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_26B1::it_8026B894(a0, a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DF34(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DF34(a0) };
    fn it_8026E32C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026E32C(a0, a1) };
    fn it_8026E414(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E414(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_8027236C(a0: *mut HSD_GObj) -> *mut HSD_GObj => unsafe { crate::tu::itcoll::it_8027236C(a0) };
    fn it_802723FC(a0: *mut HSD_GObj) -> *mut HSD_GObj => unsafe { crate::tu::itcoll::it_802723FC(a0) };
    fn it_802725D4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802725D4(a0) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272940(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272940(a0) };
    fn it_80272980(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272980(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80274C88(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274C88(a0) };
    fn it_80274CAC(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274CAC(a0) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_80275174(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275174(a0, a1) };
    fn it_80275414(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275414(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275444(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275444(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275474(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275474(&mut *((*a0).user_data as *mut Item)) };
    fn it_802754D4(a0: *mut HSD_GObj) => unsafe { crate::tu::ithitbox::it_802754D4(a0) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::tu::ithitbox::it_802756D0(a0) };
    fn it_802756E0(a0: *mut HSD_GObj) => unsafe { crate::tu::ithitbox::it_802756E0(a0) };
    fn it_8027570C(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::ithitbox::it_8027570C(a0, a1) };
    fn it_8027572C(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::ithitbox::it_8027572C(a0, a1) };
    fn it_80275D5C(a0: *mut HSD_GObj, a1: *mut _itECB) => unsafe { crate::tu::itmaplib::it_80275D5C(a0, a1 as _) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::tu::itmaplib::it_802762BC(a0) };
    fn it_80276CB8(a0: *mut HSD_GObj) => unsafe { crate::tu::itmaplib::it_80276CB8(a0) };
    fn it_80277040(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itmaplib::it_80277040(a0) as _ };
    fn it_8027770C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itmaplib::it_8027770C(a0) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::it_3F14::it_804D6D28;
}
// END GENERATED EXT

crate::jobj_inlines_it!();
crate::inl_jobj_set!();
crate::inl_Item_UpdateRollingShellRotation!();

/// `typedef struct itGShell_Attrs` (64 bytes).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct itGShell_Attrs {
    pub x0: f32,
    pub x4: f32,
    pub x8: f32,
    pub xC: f32,
    pub x10: f32,
    pub x14: f32,
    pub pad18: [i8; 4],
    pub x1C: f32,
    pub x20: f32,
    pub x24: f32,
    pub x28: f32,
    pub x2C: f32,
    pub x30: f32,
    pub x34: Vec3,
}
impl itGShell_Attrs {
    #[inline]
    pub fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}

/// `ItemStateTable it_803F5BA8[10]`
pub static it_803F5BA8: [ItemStateTable; 10] = [
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itGshell_UnkMotion0_Anim), physics_updated: Some(itGshell_UnkMotion0_Phys), collided: Some(itGshell_UnkMotion0_Coll) },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itGshell_UnkMotion1_Anim), physics_updated: Some(itGshell_UnkMotion1_Phys), collided: Some(itGshell_UnkMotion1_Coll) },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itGshell_UnkMotion2_Anim), physics_updated: Some(itGshell_UnkMotion2_Phys), collided: None },
    ItemStateTable { anim_id: 0, animated: Some(itGshell_UnkMotion3_Anim), physics_updated: Some(itGshell_UnkMotion3_Phys), collided: Some(itGshell_UnkMotion3_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itGshell_UnkMotion4_Anim), physics_updated: Some(itGshell_UnkMotion4_Phys), collided: Some(itGshell_UnkMotion4_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itGshell_UnkMotion6_Anim), physics_updated: Some(itGshell_UnkMotion6_Phys), collided: Some(itGshell_UnkMotion6_Coll) },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itGshell_UnkMotion6_Anim), physics_updated: Some(itGshell_UnkMotion6_Phys), collided: Some(itGshell_UnkMotion6_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itGshell_UnkMotion8_Anim), physics_updated: Some(itGshell_UnkMotion8_Phys), collided: Some(itGshell_UnkMotion8_Coll) },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itGshell_UnkMotion8_Anim), physics_updated: Some(itGshell_UnkMotion8_Phys), collided: Some(itGshell_UnkMotion8_Coll) },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itGshell_UnkMotion9_Anim), physics_updated: Some(itGshell_UnkMotion9_Phys), collided: Some(itGshell_UnkMotion9_Coll) },
];

/// `ABS(x)`: `x < 0 ? -x : x` (keeps `-0.0` and NaN as they are).
#[inline]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

#[inline]
unsafe fn attrs_of(ip: *mut Item) -> *mut itGShell_Attrs {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itGShell_Attrs
}

pub unsafe extern "C" fn it_8028B8D8(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8028B8D8");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    if (*ip).xDD4_itemVar.gshell.xDDC <= 0.0 {
        let jobj = GET_JOBJ(gobj);
        let mut v: Vec3 = (*attrs).x34;
        let temp = -(*ip).facing_dir;
        v.x *= temp;
        ext::efAsync_Spawn(gobj, &raw mut (*GET_ITEM(gobj)).xBC0 as *mut c_void, 2, 1029, jobj, &raw mut v);
        (*ip).xDD4_itemVar.gshell.xDDC = (*attrs).x30;
    } else {
        (*ip).xDD4_itemVar.gshell.xDDC -= 1.0;
    }
}

pub unsafe extern "C" fn it_8028B988(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8028B988");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    if (*ip).xDD4_itemVar.gshell.xDEC_b1() != 0 {
        (*ip).xDD4_itemVar.gshell.xDE0 -= 1.0;
        if (*ip).xDD4_itemVar.gshell.xDE0 <= 0.0 {
            (*ip).xDD4_itemVar.gshell.set_xDEC_b1(0);
            (*ip).xDD4_itemVar.gshell.xDE0 = (*attrs).x28;
            ext::it_8027572C(gobj, 0);
            if (*ip).xDD4_itemVar.gshell.xDEC_b2() != 0 {
                ext::it_802756E0(gobj);
            }
        }
    }
}

pub unsafe extern "C" fn it_8028BA2C(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8028BA2C");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    ext::it_80275D5C(gobj, &raw mut (*ip).xC0C);
    if abs_c((*ip).x40_vel.x) < (*attrs).x8 {
        (*ip).x40_vel.x = 0.0;
        (*ip).x40_vel.y = 0.0;
        (*ip).x40_vel.z = 0.0;
        if ext::it_80277040(gobj) == 0 {
            it_8028BE54(gobj);
        } else {
            itGShell_Logic14_EnteredAir(gobj);
        }
    } else {
        it_8028C3A8(gobj);
    }
}

pub unsafe extern "C" fn it_8028BAD8(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8028BAD8");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    match (*ip).msid as i32 {
        0 | 1 | 3 | 4 | 9 => {
            (*ip).x40_vel.x = -(*ip).xCCC_incDamageDirection * ((*ip).xCA0 as f32 * (*attrs).x14);
            (*ip).xDD4_itemVar.gshell.set_xDEC_b2(1);
        }
        5 | 6 | 7 | 8 => {
            (*ip).x40_vel.x = fmadds(-(*ip).xCCC_incDamageDirection, (*ip).xCA0 as f32 * (*attrs).x14, (*ip).x40_vel.x);
            (*ip).xDD4_itemVar.gshell.set_xDEC_b2(1);
        }
        _ => {}
    }
    ext::it_8027236C(gobj);
    ext::it_802756D0(gobj);
    if (*ip).ground_or_air == GA_Air as _ {
        it_8028C898(gobj);
    } else {
        it_8028C3A8(gobj);
    }
    (*ip).xDD4_itemVar.gshell.set_xDEC_b1(1);
    ext::it_8027570C(gobj, 0);
}

pub unsafe extern "C" fn it_8028BC2C(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8028BC2C");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    match (*ip).msid as i32 {
        5 | 6 | 7 | 8 => {
            ext::it_802756D0(gobj);
            ext::it_80275444(gobj);
            (*ip).x40_vel.x = -(*ip).x40_vel.x * (*attrs).xC * ext::HSD_Randf();
            (*ip).x40_vel.y = (*attrs).x10;
            if (*ip).xDD4_itemVar.gshell.xDEC_b0() != 0 {
                let mut rand_max: u8 = (*ext::it_804D6D28).x48_byte;
                rand_max &= 0xF;
                (*ip).xDD4_itemVar.gshell.xDE8 = ext::HSD_Randi(rand_max as i32);
            } else {
                (*ip).xDD4_itemVar.gshell.set_xDEC_b0(1);
            }
            ext::it_802762BC(ip);
            it_8028C018(gobj);
            (*ip).xDD4_itemVar.gshell.set_xDEC_b3(1);
            (*ip).xDD4_itemVar.gshell.xDE4 = (*attrs).x2C;
            (*ip).xDD4_itemVar.gshell.set_xDEC_b2(1);
        }
        0 | 1 | 3 | 4 | 9 => {
            (*ip).x40_vel.x = (*attrs).x1C * -(*ip).xCD0;
            ext::it_802723FC(gobj);
            (*ip).xDD4_itemVar.gshell.set_xDEC_b2(1);
            if (*ip).ground_or_air == GA_Air as _ {
                it_8028C898(gobj);
            } else {
                it_8028C3A8(gobj);
            }
        }
        _ => {}
    }
    ext::it_802756D0(gobj);
    (*ip).xDD4_itemVar.gshell.set_xDEC_b1(1);
    ext::it_8027570C(gobj, 0);
}

pub unsafe extern "C" fn itGShell_Logic14_Spawned(gobj: *mut HSD_GObj) {
    melee_core::ct!("itGShell_Logic14_Spawned");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    (*ip).xDD4_itemVar.gshell.xDD4 = (*attrs).x0;
    ext::it_80275174(gobj, (*ip).xDD4_itemVar.gshell.xDD4);
    (*ip).xDD4_itemVar.gshell.xDE8 = 1;
    (*ip).xDD4_itemVar.gshell.set_xDEC_b0(0);
    (*ip).xDD4_itemVar.gshell.set_xDEC_b1(0);
    (*ip).xDD4_itemVar.gshell.xDE0 = (*attrs).x28;
    (*ip).xDD4_itemVar.gshell.set_xDEC_b3(0);
    (*ip).xDD4_itemVar.gshell.xDE4 = (*attrs).x2C;
    (*ip).xDD4_itemVar.gshell.set_xDEC_b2(1);
    it_8028C018(gobj);
}

pub unsafe extern "C" fn it_8028BE54(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8028BE54");
    let ip = GET_ITEM(gobj);
    ext::it_8026B390(gobj);
    ext::it_80275444(gobj);
    ext::it_802754D4(gobj);
    ext::it_802756E0(gobj);
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.z = 0.0;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
    (*ip).xDD4_itemVar.gshell.set_xDEC_b3(0);
    ext::it_80274CAC(gobj);
    (*ip).jumped_on = Some(it_8028CF68);
}

pub unsafe extern "C" fn itGshell_UnkMotion0_Anim(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGshell_UnkMotion0_Anim");
    0
}

pub unsafe extern "C" fn itGshell_UnkMotion0_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itGshell_UnkMotion0_Phys");}

pub unsafe extern "C" fn itGshell_UnkMotion0_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGshell_UnkMotion0_Coll");
    ext::it_8026D62C(gobj, Some(it_8028C018));
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let attrs = attrs_of(ip);
    if (*ip).ground_or_air == GA_Ground as _ {
        Item_UpdateRollingShellRotation(gobj, ip, jobj, &raw const (*attrs).x20);
    }
    0
}

pub unsafe extern "C" fn it_8028C018(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8028C018");
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
    ext::it_80274C88(gobj);
}

pub unsafe extern "C" fn itGshell_UnkMotion1_Anim(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGshell_UnkMotion1_Anim");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.gshell.xDEC_b3() != 0 && (*ip).xDD0_flag.x0.b0() != 1 {
        if (*ip).xDD4_itemVar.gshell.xDE4 <= 0.0 {
            ext::it_80274CAC(gobj);
            (*ip).jumped_on = Some(it_8028CF68);
        } else {
            (*ip).xDD4_itemVar.gshell.xDE4 -= 1.0;
        }
    }
    0
}

pub unsafe extern "C" fn itGshell_UnkMotion1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itGshell_UnkMotion1_Phys");
    let ip = GET_ITEM(gobj);
    let attr = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itGshell_UnkMotion1_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGshell_UnkMotion1_Coll");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.gshell.xDE8 != 0 {
        ext::it_8026E32C(gobj, Some(it_8028BE54));
    } else {
        return ext::it_8026DF34(gobj);
    }
    0
}

pub unsafe extern "C" fn itGShell_Logic14_PickedUp(gobj: *mut HSD_GObj) {
    melee_core::ct!("itGShell_Logic14_PickedUp");
    let ip = GET_ITEM(gobj);
    ext::it_80275474(gobj);
    (*ip).xDD4_itemVar.gshell.set_xDEC_b3(0);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itGshell_UnkMotion2_Anim(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGshell_UnkMotion2_Anim");
    0
}

pub unsafe extern "C" fn itGshell_UnkMotion2_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itGshell_UnkMotion2_Phys");}

pub unsafe extern "C" fn itGShell_Logic14_Thrown(gobj: *mut HSD_GObj) {
    melee_core::ct!("itGShell_Logic14_Thrown");
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.gshell.xDE8 = 1;
    ext::Item_80268E5C(gobj, 3, 6);
    ext::Item_8026AE84(ip, 242, 127, 64);
}

pub unsafe extern "C" fn itGshell_UnkMotion3_Anim(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGshell_UnkMotion3_Anim");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    if (*ip).xDD4_itemVar.gshell.xDEC_b1() != 0 {
        (*ip).xDD4_itemVar.gshell.xDE0 -= 1.0;
        if (*ip).xDD4_itemVar.gshell.xDE0 <= 0.0 {
            (*ip).xDD4_itemVar.gshell.set_xDEC_b1(0);
            (*ip).xDD4_itemVar.gshell.xDE0 = (*attrs).x28;
            ext::it_8027572C(gobj, 0);
            if (*ip).xDD4_itemVar.gshell.xDEC_b2() != 0 {
                ext::it_802756E0(gobj);
            }
        }
    }
    0
}

pub unsafe extern "C" fn itGshell_UnkMotion3_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itGshell_UnkMotion3_Phys");
    let ip = GET_ITEM(gobj);
    let attr = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itGshell_UnkMotion3_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGshell_UnkMotion3_Coll");
    ext::it_8026E414(gobj, Some(it_8028BA2C));
    0
}

pub unsafe extern "C" fn itGShell_Logic14_Dropped(gobj: *mut HSD_GObj) {
    melee_core::ct!("itGShell_Logic14_Dropped");
    ext::Item_80268E5C(gobj, 4, 6);
}

pub unsafe extern "C" fn itGshell_UnkMotion4_Anim(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGshell_UnkMotion4_Anim");
    it_8028B988(gobj);
    0
}

pub unsafe extern "C" fn itGshell_UnkMotion4_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itGshell_UnkMotion4_Phys");
    let ip = GET_ITEM(gobj);
    let attr = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itGshell_UnkMotion4_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGshell_UnkMotion4_Coll");
    ext::it_8026E414(gobj, Some(it_8028BA2C));
    0
}

pub unsafe extern "C" fn it_8028C3A8(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8028C3A8");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    ext::it_8026B3A8(gobj);
    ext::it_80275474(gobj);
    ext::it_80275414(gobj);
    ext::it_80275158(gobj, (*ext::it_804D6D28).x30_lifetime as f32);
    (*ip).xD5C = 0;
    (*ip).xDD4_itemVar.gshell.xDD8 = (*attrs).x24;
    (*ip).xDD4_itemVar.gshell.xDDC = (*attrs).x30;
    (*ip).xDD4_itemVar.gshell.set_xDEC_b3(0);
    if abs_c((*ip).x40_vel.x) > (*attrs).x4 {
        let dir: i32 = if (*ip).x40_vel.x < 0.0 { -1 } else { 1 };
        (*ip).x40_vel.x = dir as f32;
        (*ip).x40_vel.x *= (*attrs).x4;
    }
    ext::it_80272980(gobj);
    if abs_c((*ip).x40_vel.x) < (*attrs).x8 {
        ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE as u32);
        ext::it_8026B390(gobj);
        ext::it_802754D4(gobj);
    } else {
        ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE as u32);
    }
    ext::it_80274CAC(gobj);
    (*ip).jumped_on = Some(it_8028CF68);
}

pub unsafe extern "C" fn itGshell_UnkMotion6_Anim(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGshell_UnkMotion6_Anim");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.gshell.xDD8 <= 0.0 {
        if (*ip).xDCD_flag.x0.b5() == 0 {
            ext::it_80275444(gobj);
        }
    } else {
        (*ip).xDD4_itemVar.gshell.xDD8 -= 1.0;
    }
    it_8028B988(gobj);
    if (*ip).msid == 6 || (*ip).msid == 5 {
        it_8028B8D8(gobj);
    }
    if (*ip).xDD4_itemVar.gshell.xDD4 <= 0.0 {
        return 1;
    }
    (*ip).xDD4_itemVar.gshell.xDD4 -= 1.0;
    0
}

pub unsafe extern "C" fn itGshell_UnkMotion6_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itGshell_UnkMotion6_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    if abs_c((*ip).x40_vel.x) < (*attrs).x8 && (*ip).xDC8_word.flags.x15() == 0 {
        ext::it_8026B390(gobj);
        ext::it_802725D4(gobj);
        ext::it_802754D4(gobj);
    }
}

pub unsafe extern "C" fn itGshell_UnkMotion6_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGshell_UnkMotion6_Coll");
    ext::it_8026D62C(gobj, Some(it_8028C898));
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let attrs = attrs_of(ip);
    if (*ip).ground_or_air == GA_Ground as _ {
        Item_UpdateRollingShellRotation(gobj, ip, jobj, &raw const (*attrs).x20);
    }
    if ext::it_8027770C(gobj) != 0 {
        ext::it_80272980(gobj);
    }
    0
}

pub unsafe extern "C" fn it_8028C898(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8028C898");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    ext::it_8026B3A8(gobj);
    ext::it_80275474(gobj);
    ext::it_80275414(gobj);
    ext::it_80275158(gobj, (*ext::it_804D6D28).x30_lifetime as f32);
    (*ip).xD5C = 0;
    (*ip).xDD4_itemVar.gshell.xDD8 = (*attrs).x24;
    (*ip).xDD4_itemVar.gshell.xDDC = (*attrs).x30;
    (*ip).xDD4_itemVar.gshell.set_xDEC_b3(0);
    if abs_c((*ip).x40_vel.x) > (*attrs).x4 {
        let dir: i32 = if (*ip).x40_vel.x < 0.0 { -1 } else { 1 };
        (*ip).x40_vel.x = dir as f32;
        (*ip).x40_vel.x *= (*attrs).x4;
    }
    ext::it_80272980(gobj);
    if abs_c((*ip).x40_vel.x) < (*attrs).x8 {
        ext::Item_80268E5C(gobj, 8, ITEM_ANIM_UPDATE as u32);
        ext::it_8026B390(gobj);
    } else {
        ext::Item_80268E5C(gobj, 7, ITEM_ANIM_UPDATE as u32);
    }
    ext::it_80274CAC(gobj);
    (*ip).jumped_on = Some(it_8028CF68);
}

pub unsafe extern "C" fn itGshell_UnkMotion8_Anim(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGshell_UnkMotion8_Anim");
    let ip = (*gobj).user_data as *mut Item;
    if (*ip).xDD4_itemVar.gshell.xDD8 <= 0.0 {
        if (*ip).xDCD_flag.x0.b5() == 0 {
            ext::it_80275444(gobj);
        }
    } else {
        (*ip).xDD4_itemVar.gshell.xDD8 -= 1.0;
    }
    it_8028B988(gobj);
    if (*ip).msid == 6 || (*ip).msid == 5 {
        it_8028B8D8(gobj);
    }
    if (*ip).xDD4_itemVar.gshell.xDD4 <= 0.0 {
    } else {
        (*ip).xDD4_itemVar.gshell.xDD4 -= 1.0;
    }
    0
}

pub unsafe extern "C" fn itGshell_UnkMotion8_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itGshell_UnkMotion8_Phys");
    let ip = GET_ITEM(gobj);
    let attr = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itGshell_UnkMotion8_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGshell_UnkMotion8_Coll");
    ext::it_8026E414(gobj, Some(it_8028BA2C));
    0
}

pub unsafe extern "C" fn itGShell_Logic14_EnteredAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("itGShell_Logic14_EnteredAir");
    let ip = GET_ITEM(gobj);
    (*ip).xDC8_word.flags.set_x1F(1);
    (*ip).xDD4_itemVar.gshell.set_xDEC_b3(0);
    ext::Item_80268E5C(gobj, 9, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itGshell_UnkMotion9_Anim(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGshell_UnkMotion9_Anim");
    0
}

pub unsafe extern "C" fn itGshell_UnkMotion9_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itGshell_UnkMotion9_Phys");}

pub unsafe extern "C" fn itGshell_UnkMotion9_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGshell_UnkMotion9_Coll");
    ext::it_8026E8C4(gobj, Some(it_8028BE54), Some(it_8028C018));
    let ip = GET_ITEM(gobj);
    let mut jobj = GET_JOBJ(gobj);
    let attrs = attrs_of(ip);
    if (*ip).ground_or_air == GA_Ground as _ {
        ext::it_80276CB8(gobj);
        jobj = HSD_JObjGetChild(jobj);
        HSD_JObjAddRotationY(jobj, (*attrs).x20 * abs_c((*ip).x40_vel.x));
    }
    0
}

pub unsafe extern "C" fn itGShell_Logic14_DmgDealt(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGShell_Logic14_DmgDealt");
    ext::it_80272940(gobj);
    0
}

pub unsafe extern "C" fn itGShell_Logic14_DmgReceived(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGShell_Logic14_DmgReceived");
    ext::it_80272940(gobj);
    it_8028BAD8(gobj);
    0
}

pub unsafe extern "C" fn itGShell_Logic14_Reflected(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGShell_Logic14_Reflected");
    let ip = GET_ITEM(gobj);
    ext::it_80273030(gobj);
    (*ip).xDD4_itemVar.gshell.xDD4 = (*ip).xD48_halfLifeTimer;
    0
}

/// `static inline void shellHit(HSD_GObj*)`
pub unsafe extern "C" fn shellHit(gobj: *mut HSD_GObj) {
    melee_core::ct!("shellHit");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    ext::it_802756D0(gobj);
    ext::it_80275444(gobj);
    (*ip).x40_vel.x = -(*ip).x40_vel.x * (*attrs).xC * ext::HSD_Randf();
    (*ip).x40_vel.y = (*attrs).x10;
    if (*ip).xDD4_itemVar.gshell.xDEC_b0() != 0 {
        let mut rand_max: u8 = (*ext::it_804D6D28).x48_byte;
        rand_max &= 0xF;
        (*ip).xDD4_itemVar.gshell.xDE8 = ext::HSD_Randi(rand_max as i32);
    } else {
        (*ip).xDD4_itemVar.gshell.set_xDEC_b0(1);
    }
    ext::it_802762BC(ip);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
    ext::it_80274C88(gobj);
    (*ip).xDD4_itemVar.gshell.set_xDEC_b3(1);
    (*ip).xDD4_itemVar.gshell.xDE4 = (*attrs).x2C;
}

pub unsafe extern "C" fn itGShell_Logic14_Clanked(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGShell_Logic14_Clanked");
    shellHit(gobj);
    0
}

pub unsafe extern "C" fn itGShell_Logic14_HitShield(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGShell_Logic14_HitShield");
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 3 || (*ip).msid == 4 {
        ext::itColl_BounceOffVictim(gobj);
    } else if ((*ip).msid as u32).wrapping_sub(5) <= 3 {
        shellHit(gobj);
    }
    0
}

pub unsafe extern "C" fn itGShell_Logic14_ShieldBounced(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGShell_Logic14_ShieldBounced");
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 3 || (*ip).msid == 4 {
        ext::itColl_BounceOffShield(gobj);
    }
    0
}

pub unsafe extern "C" fn it_8028CF68(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("it_8028CF68");
    let ip = GET_ITEM(gobj);
    ext::it_80272940(gobj);
    it_8028BC2C(gobj);
    ext::Item_8026AE84(ip, 241, 127, 64);
    0
}

pub unsafe extern "C" fn itGShell_Logic14_EvtUnk(gobj: *mut HSD_GObj, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("itGShell_Logic14_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}

// ---- folded from itkinds.rs (consolidation steps 21/22) ----
/// `it_8028B988` (itgshell.c): the body of `itGshell_UnkMotion3_Anim` without the return.
/// # Safety
/// The special attributes must be a valid `itGShell_Attrs_itkinds`.
pub unsafe fn it_8028b988(ip: &mut Item) {
    melee_core::ct!("it_8028b988");
    let attrs = sa::<itGShell_Attrs_itkinds>(ip);
    if ip.xDD4_itemVar.gshell.xDEC_b1() != 0 {
        ip.xDD4_itemVar.gshell.xDE0 -= 1.0;
        if ip.xDD4_itemVar.gshell.xDE0 <= 0.0 {
            ip.xDD4_itemVar.gshell.set_xDEC_b1(0);
            ip.xDD4_itemVar.gshell.xDE0 = attrs.x28;
            it_8027572c(ip, 0);
            if ip.xDD4_itemVar.gshell.xDEC_b2() != 0 {
                it_802756e0(ip);
            }
        }
    }
}
