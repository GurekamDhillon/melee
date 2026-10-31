//! Port of src/melee/it/kinds/itwhispyapple.c: Whispy Woods' apples (logic table 18). Callees of unported
//! TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction)
//! by tests/kinds_itwhispyapple.rs.
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
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjClearFlagsAll(a0, a1) };
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::itColl_BounceOffVictim(a0) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BD24(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd24(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026BDB4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BDB4(a0) };
    fn it_8026D3CC() -> i32 => unsafe { crate::tu::itspawn::it_8026D3CC() as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DAA8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DAA8(a0) };
    fn it_8026DBC8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DBC8(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_802725D4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802725D4(a0) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_802728C8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802728C8(a0) };
    fn it_80272C08(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272C08(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_8027429C(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_2725::it_8027429C(a0, a1 as _) };
    fn it_80274ED8() => unsafe { crate::tu::it_2725::it_80274ED8() };
    fn it_802750F8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802750F8(a0) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_80275444(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275444(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_80276FC4(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::itmaplib::it_80276FC4(a0, a1) };
    fn lb_800119DC(a0: *mut Vec, a1: i32, a2: f32, a3: f32, a4: f32) => unsafe { melee_runtime::tu::lb_00F9::lb_800119DC(a0, a1, a2, a3, a4) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::it_3F14::it_804D6D28;
}
// END GENERATED EXT


/// `ItemStateTable it_803F91D0[]`
pub static mut it_803F91D0: [ItemStateTable; 8] = [
    st_row(1, pred_b(itWhispyapple_UnkMotion0_Anim), Some(itWhispyapple_UnkMotion0_Phys), pred_b(itWhispyapple_UnkMotion0_Coll)),
    st_row(0, pred_b(itWhispyapple_UnkMotion1_Anim), Some(itWhispyapple_UnkMotion1_Phys), pred_b(itWhispyapple_UnkMotion1_Coll)),
    st_row(0, pred_b(itWhispyapple_UnkMotion5_Anim), Some(itWhispyapple_UnkMotion5_Phys), pred_b(itWhispyapple_UnkMotion5_Coll)),
    st_row(0, pred_b(itWhispyapple_UnkMotion3_Anim), Some(itWhispyapple_UnkMotion3_Phys), None),
    st_row(1, pred_b(itWhispyapple_UnkMotion5_Anim), Some(itWhispyapple_UnkMotion5_Phys), pred_b(itWhispyapple_UnkMotion5_Coll)),
    st_row(1, pred_b(itWhispyapple_UnkMotion5_Anim), Some(itWhispyapple_UnkMotion5_Phys), pred_b(itWhispyapple_UnkMotion5_Coll)),
    st_row(0, pred_b(itWhispyapple_UnkMotion6_Anim), Some(itWhispyapple_UnkMotion6_Phys), pred_b(itWhispyapple_UnkMotion6_Coll)),
    st_row(2, pred_b(itWhispyapple_UnkMotion7_Anim), Some(itWhispyapple_UnkMotion7_Phys), pred_b(itWhispyapple_UnkMotion7_Coll)),
];

/// `static s32 it_803F9250[4]`
pub static mut it_803F9250: [i32; 4] = [0x68FB4, 0x68FB5, 0x68FB6, 0];

/// `ABS(x)` (Runtime/platform.h) for f32.
#[inline]
fn ABS(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `itResetVelocity` comes from `kinds_inl`.
pub unsafe extern "C" fn itWhispyapple_UnkMotion0_Anim_inline(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhispyapple_UnkMotion0_Anim_inline");
    let jobj = GET_JOBJ(gobj);
    let ip = GET_ITEM(gobj);
    let child = HSD_JObjGetChild(jobj);
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).msid == 7 {
        ext::HSD_JObjSetFlagsAll(child, JOBJ_HIDDEN as u32);
    } else if ((*ip).xD44_lifeTimer <= (*ext::it_804D6D28).x34 as f32) || (ABS((*ip).pos.z) > 1.0) {
        ext::it_802728C8(gobj);
    } else {
        ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    }
    if (*ip).xD44_lifeTimer <= 0.0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn it_802EE200(parent: *mut HSD_GObj, pos: *mut Vec3, threshold1: f32, mut threshold2: f32) -> *mut Item_GObj {
    melee_core::ct!("it_802EE200");
    let mut spawn: SpawnItem = core::mem::zeroed();
    let mut rand = ext::HSD_Randf();
    let facing_dir: f32;

    spawn.pos = *pos;
    spawn.prev_pos = spawn.pos;
    if ext::HSD_Randi(2) != 0 {
        facing_dir = 1.0;
    } else {
        facing_dir = -1.0;
    }
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = null_mut();
    spawn.x44_flag.anon_0.set_b0(0);
    spawn.x40 = 0;
    if ext::it_8026D3CC() == 0 {
        rand -= threshold2;
        if rand < 0.0 {
            rand = 0.0;
        }
        threshold2 = 0.0;
        spawn.kind = It_Kind_WhispyApple;
    } else if rand < threshold2 {
        spawn.kind = It_Kind_WhispyHealApple;
    } else {
        spawn.kind = It_Kind_WhispyApple;
    }
    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        let ip = (*gobj).user_data as *mut Item;
        ext::it_80274ED8();
        it_802EE374(gobj);
        if rand < (threshold1 + threshold2) {
            (*ip).xDD4_itemVar.whispyapple.xDD8 = 0;
        } else {
            (*ip).xDD4_itemVar.whispyapple.xDD8 = 1;
            ext::it_8026B3A8(gobj);
        }
        ext::it_802750F8(gobj);
    }
    gobj
}

pub unsafe extern "C" fn it_802EE374(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EE374");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itWhispyAppleAttributes;

    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    ext::it_80275158(gobj, (*attrs).x8 as f32);
    (*ip).xDD4_itemVar.whispyapple.xDD4_heal = (*attrs).x4;
    let rand = ext::HSD_Randf();
    // `2.0f * x18 * rand - x18`: one fused multiply-subtract (shipping contraction): (2*x18) rounded to f32 first
    let two_x18 = 2.0f32 * (*attrs).x18;
    (*ip).x40_vel.x = (two_x18 as f64 * rand as f64 + (-(*attrs).x18) as f64) as f32;
    (*ip).x40_vel.y = (*attrs).x14;
    (*ip).x40_vel.z = ABS((*ip).pos.z / 20.0);
}

pub unsafe extern "C" fn itWhispyapple_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhispyapple_UnkMotion0_Anim");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.whispyapple.xDD8 == 1 {
        ext::it_8026B3A8(gobj);
    }
    itWhispyapple_UnkMotion0_Anim_inline(gobj)
}

pub unsafe extern "C" fn itWhispyapple_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itWhispyapple_UnkMotion0_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;

    if (*ip).xDD4_itemVar.whispyapple.xDD8 == 1 {
        ext::it_8026B3A8(gobj);
    }
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

#[inline]
unsafe fn itWhispyapple_UnkMotion0_Coll_inline(gobj: *mut Item_GObj) {
    let ip = (*gobj).user_data as *mut Item;
    let coll = ext::it_8026DAA8(gobj);
    if coll & 0xF != 0 {
        let r = ext::HSD_Randi(3);
        ext::Item_8026AE84(ip, it_803F9250[r as usize], 0x7F, 0x40);
        ext::it_80276FC4(gobj, coll);
        if (coll & 1) != 0 && ext::it_8026DBC8(gobj) != 0 {
            it_802EE6A0(gobj);
        }
    }
}

pub unsafe extern "C" fn itWhispyapple_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhispyapple_UnkMotion0_Coll");
    let ip = (*gobj).user_data as *mut Item;

    if (*ip).xDD4_itemVar.whispyapple.xDD8 == 1 {
        ext::it_8026B3A8(gobj);
    }
    if (*ip).pos.z >= 0.0 {
        (*ip).pos.z = 0.0;
        (*ip).x40_vel.z = 0.0;
        if (*ip).xDD4_itemVar.whispyapple.xDD8 == 1 {
            ext::it_8026E15C(gobj, Some(it_802EED00));
        } else {
            itWhispyapple_UnkMotion0_Coll_inline(gobj);
        }
    }
    false
}

pub unsafe extern "C" fn it_802EE6A0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EE6A0");
    let ip = GET_ITEM(gobj);
    let val: f32 = 0.0;
    (*ip).x40_vel.x = val;
    (*ip).x40_vel.y = val;
    (*ip).x40_vel.z = val;
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itWhispyapple_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhispyapple_UnkMotion1_Anim");
    itWhispyapple_UnkMotion0_Anim_inline(gobj)
}

pub unsafe extern "C" fn itWhispyapple_UnkMotion1_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itWhispyapple_UnkMotion1_Phys");}

pub unsafe extern "C" fn itWhispyapple_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhispyapple_UnkMotion1_Coll");
    ext::it_8026D62C(gobj, Some(fn_802EE7FC));
    false
}

pub unsafe extern "C" fn fn_802EE7FC(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_802EE7FC");
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itWhispyapple_UnkMotion5_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhispyapple_UnkMotion5_Anim");
    itWhispyapple_UnkMotion0_Anim_inline(gobj)
}

pub unsafe extern "C" fn itWhispyapple_UnkMotion5_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itWhispyapple_UnkMotion5_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itWhispyapple_UnkMotion5_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhispyapple_UnkMotion5_Coll");
    let ip = GET_ITEM(gobj);

    let coll = ext::it_8026DAA8(gobj);
    if coll & 0xF != 0 {
        let r = ext::HSD_Randi(3);
        ext::Item_8026AE84(ip, it_803F9250[r as usize], 0x7F, 0x40);
        ext::it_80276FC4(gobj, coll);
        if (coll & 1) != 0 && ext::it_8026DBC8(gobj) != 0 {
            it_802EE6A0(gobj);
        }
    }
    false
}

pub unsafe extern "C" fn it_802EEA08(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EEA08");
    let jobj = GET_JOBJ(gobj);
    let child = HSD_JObjGetChild(jobj);
    ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itWhispyapple_UnkMotion3_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhispyapple_UnkMotion3_Anim");
    false
}

pub unsafe extern "C" fn itWhispyapple_UnkMotion3_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itWhispyapple_UnkMotion3_Phys");}

pub unsafe extern "C" fn it_802EEA70(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EEA70");
    let ip = GET_ITEM(gobj);
    let child = HSD_JObjGetChild(GET_JOBJ(gobj));
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    ext::it_8026B390(gobj);
    ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).pos.z = 0.0;
    ext::it_8027429C(gobj, &mut pos);
}

pub unsafe extern "C" fn it_802EEB28(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EEB28");
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let child = HSD_JObjGetChild(jobj);

    ext::it_8026B390(gobj);
    ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
    (*ip).x40_vel.z = 0.0;
    (*ip).pos.z = 0.0;
}

pub unsafe extern "C" fn itWhispyApple_Logic18_EnteredAir(gobj: *mut Item_GObj) {
    melee_core::ct!("itWhispyApple_Logic18_EnteredAir");
    ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itWhispyapple_UnkMotion6_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhispyapple_UnkMotion6_Anim");
    itWhispyapple_UnkMotion0_Anim_inline(gobj)
}

pub unsafe extern "C" fn itWhispyapple_UnkMotion6_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itWhispyapple_UnkMotion6_Phys");}

pub unsafe extern "C" fn itWhispyapple_UnkMotion6_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhispyapple_UnkMotion6_Coll");
    ext::it_8026E8C4(gobj, Some(it_802EE6A0), Some(fn_802EE7FC));
    false
}

pub unsafe extern "C" fn it_802EED00(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EED00");
    let jobj = GET_JOBJ(gobj);
    let ip = GET_ITEM(gobj);
    let mut pos: Vec3;

    ext::it_8026B3A8(gobj);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    ext::it_8026BD24(gobj);
    ext::it_80275158(gobj, 20.0);
    ext::it_8026BDB4(gobj);
    itResetVelocity(ip);
    ext::it_80272C08(gobj);
    ext::it_802756D0(gobj);
    ext::it_80275444(gobj);
    pos = (*ip).pos;
    ext::lb_800119DC(&mut pos, 0x78, 1.0, 0.02, 1.0471976);
    ext::Item_80268E5C(gobj, 7, 0x12);
}

pub unsafe extern "C" fn itWhispyapple_UnkMotion7_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhispyapple_UnkMotion7_Anim");
    itWhispyapple_UnkMotion0_Anim_inline(gobj)
}

pub unsafe extern "C" fn itWhispyapple_UnkMotion7_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itWhispyapple_UnkMotion7_Phys");}

pub unsafe extern "C" fn itWhispyapple_UnkMotion7_Coll(_item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhispyapple_UnkMotion7_Coll");
    false
}

pub unsafe extern "C" fn it_802EEED0(_item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802EEED0");
    true
}

pub unsafe extern "C" fn it_802EEED8(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802EEED8");
    ext::it_802725D4(gobj);
    ext::itColl_BounceOffVictim(gobj);
    false
}

pub unsafe extern "C" fn it_802EEF10(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802EEF10");
    ext::it_80273030(gobj) != 0
}

pub unsafe extern "C" fn it_802EEF30(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802EEF30");
    ext::it_802725D4(gobj);
    ext::itColl_BounceOffVictim(gobj);
    false
}

pub unsafe extern "C" fn it_802EEF68(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802EEF68");
    ext::itColl_BounceOffShield(gobj) != 0
}

pub unsafe extern "C" fn it_802EEF88(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EEF88");
    ext::it_8026B894(gobj, ref_gobj);
}
