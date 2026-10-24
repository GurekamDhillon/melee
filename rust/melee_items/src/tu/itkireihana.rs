//! Native port of src/melee/it/kinds/itkireihana.c: the Kireihana (Kirei-hana / flower stage hazard
//! item) state machine: spawn `itKireihana_Logic15_Spawned`, the five states (air / ground, timers,
//! fall speed) and the state table `it_803F7E28`. Callees in other TUs go through `ext` (hooks;
//! unset panics "TODO"). Checked against the C (the whole TU, shipping contraction) by
//! tests/tu_itkireihana.rs.
//!
//! Header inlines compiled into the TU: `Item_SetEffectHitlagCallbacks` (it/inlines.h),
//! `Item_EnterAirStateWithHitlagAndStateDesc` (kinds/inlines.h), `GET_ITEM`. No contracted float site.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

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
    fn HSD_Randi(a0: i32) -> i32;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8);
    fn efLib_PauseAll(a0: *mut HSD_GObj);
    fn efLib_ResumeAll(a0: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32;
    fn it_8026E248(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32;
    fn it_80273454(a0: *mut HSD_GObj);
    fn it_80273670(a0: *mut HSD_GObj, a1: i32, a2: f32);
    fn it_802762BC(a0: *mut Item);
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32);
    fn it_80279FF8(a0: *mut HSD_GObj);
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32;
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32;
    fn it_8027A160(a0: *mut HSD_JObj, a1: *mut Item);
    fn it_8027A344(a0: *mut HSD_GObj);
    fn it_8027ADEC(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj, a3: f32);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn efLib_PauseAll(a0: *mut HSD_GObj) { super::efLib_PauseAll(a0) }
        pub unsafe extern "C" fn efLib_ResumeAll(a0: *mut HSD_GObj) { super::efLib_ResumeAll(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

#[inline]
unsafe fn get_item(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `Item_SetEffectHitlagCallbacks` (it/inlines.h).
#[inline]
unsafe fn Item_SetEffectHitlagCallbacks(ip: *mut Item) {
    (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}

/// `Item_EnterAirStateWithHitlagAndStateDesc` (kinds/inlines.h).
#[inline]
unsafe fn Item_EnterAirStateWithHitlagAndStateDesc(gobj: *mut Item_GObj) {
    let item = get_item(gobj);
    ext::it_802762BC(item);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as _);
    Item_SetEffectHitlagCallbacks(item);
    ext::it_80273670(gobj, 0, 0.0);
}

pub static mut it_803F7E28: [ItemStateTable; 5] = [
    ItemStateTable { anim_id: 0, animated: Some(itKireihana_UnkMotion0_Anim), physics_updated: Some(itKireihana_UnkMotion0_Phys), collided: Some(itKireihana_UnkMotion0_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itKireihana_UnkMotion1_Anim), physics_updated: Some(itKireihana_UnkMotion1_Phys), collided: Some(itKireihana_UnkMotion1_Coll) },
    ItemStateTable { anim_id: 2, animated: Some(itKireihana_UnkMotion2_Anim), physics_updated: Some(itKireihana_UnkMotion2_Phys), collided: Some(itKireihana_UnkMotion2_Coll) },
    ItemStateTable { anim_id: 3, animated: Some(itKireihana_UnkMotion3_Anim), physics_updated: Some(itKireihana_UnkMotion3_Phys), collided: Some(itKireihana_UnkMotion3_Coll) },
    ItemStateTable { anim_id: 4, animated: Some(itKireihana_UnkMotion4_Anim), physics_updated: Some(itKireihana_UnkMotion4_Phys), collided: Some(itKireihana_UnkMotion4_Coll) },
];

pub unsafe extern "C" fn itKireihana_Logic15_Spawned(gobj: *mut Item_GObj) {
    let item = get_item(gobj);
    let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itkireihanaAttributes;
    (*item).facing_dir = 0.0;
    (*item).xDAC_itcmd_var0 = 0;
    (*item).xDD4_itemVar.kireihana.timer_unk0 = (*attrs).x4 as i32;
    it_802D08F0(gobj);
    ext::it_80279CDC(gobj, (*attrs).x0);
    ext::Item_8026AE84(item, 0x2725, 0x7f, 0x40);
    (*item).xDD4_itemVar.kireihana.timer_unk1 = (*attrs).x8 as i32;
    (*item).xDD4_itemVar.kireihana.timer_unk2 = (*attrs).xC as i32;
    (*item).xDD4_itemVar.kireihana.fall_speed = 0.0;
    (*item).xDD4_itemVar.kireihana.x70 = 0.0;
}

pub unsafe extern "C" fn itKireihana_Logic15_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe extern "C" fn it_802D0138(gobj: *mut HSD_GObj) {
    let item = get_item(gobj);
    ext::it_80273454(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as _);
    Item_SetEffectHitlagCallbacks(item);
    (*item).xDD4_itemVar.kireihana.fall_speed = 0.0;
}

pub unsafe extern "C" fn it_802D01A0(gobj: *mut Item_GObj) {
    let item = get_item(gobj);
    ext::Item_80268E5C(gobj, 1, (ITEM_UNK_0x1 | ITEM_HIT_PRESERVE) as _);
    Item_SetEffectHitlagCallbacks(item);
    ext::it_8027A160((*(*item).xBBC_dynamicBoneTable).bones[1], item);
    (*item).xDD4_itemVar.kireihana.fall_speed = 0.0;
}

pub unsafe fn itKireihana_UnkMotion1_Anim_inline(gobj: *mut Item_GObj) {
    let item = get_item(gobj);
    ext::it_80273454(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as _);
    Item_SetEffectHitlagCallbacks(item);
    (*item).xDD4_itemVar.kireihana.fall_speed = 0.0;
}

pub unsafe extern "C" fn itKireihana_UnkMotion1_Anim(gobj: *mut Item_GObj) -> i32 {
    let item = get_item(gobj);
    (*item).xDD4_itemVar.kireihana.x70 = (*item).xDD4_itemVar.kireihana.x8;
    ext::it_8027A160((*(*item).xBBC_dynamicBoneTable).bones[1], item);
    if ext::it_80272C6C(gobj) == 0 {
        (*item).xDD4_itemVar.kireihana.timer_unk0 = (*item).xDD4_itemVar.kireihana.timer_unk0.wrapping_sub(1);
        if (*item).xDD4_itemVar.kireihana.timer_unk0 <= 0 {
            it_802D05D8(gobj);
        } else {
            itKireihana_UnkMotion1_Anim_inline(gobj);
        }
    }
    0
}

pub unsafe extern "C" fn itKireihana_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let item = get_item(gobj);
    (*item).xDD4_itemVar.kireihana.timer_unk1 = (*item).xDD4_itemVar.kireihana.timer_unk1.wrapping_sub(1);
    if (*item).xDD4_itemVar.kireihana.timer_unk1 == 0 {
        let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itkireihanaAttributes;
        ext::it_8027ADEC(0x470, gobj, (*(*item).xBBC_dynamicBoneTable).bones[2], 2.6);
        (*item).xDD4_itemVar.kireihana.timer_unk2 = (*item).xDD4_itemVar.kireihana.timer_unk2.wrapping_sub(1);
        if (*item).xDD4_itemVar.kireihana.timer_unk2 == 0 {
            if ext::HSD_Randi(2) != 0 {
                ext::Item_8026AE84(item, 0x2726, 0x7F, 0x40);
            } else {
                ext::Item_8026AE84(item, 0x2727, 0x7F, 0x40);
            }
            (*item).xDD4_itemVar.kireihana.timer_unk2 = (*attrs).xC as i32;
        }
        (*item).xDD4_itemVar.kireihana.timer_unk1 = (*attrs).x8 as i32;
    }
    ext::it_8027A344(gobj);
}

pub unsafe extern "C" fn itKireihana_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    let item = get_item(gobj);
    if ext::it_8026DA08(gobj) == 0 && (*item).xDD4_itemVar.kireihana.x70 < 0.0 && (*item).xDD4_itemVar.kireihana.x8 > 0.0 {
        it_802D03F8(gobj);
    }
    0
}

pub unsafe extern "C" fn it_802D03F8(gobj: *mut HSD_GObj) {
    let item = get_item(gobj);
    ext::Item_80268E5C(gobj, 2, (ITEM_UNK_0x1 | ITEM_HIT_PRESERVE) as _);
    Item_SetEffectHitlagCallbacks(item);
    ext::it_8027A160((*(*item).xBBC_dynamicBoneTable).bones[1], item);
}

pub unsafe fn itKireihana_UnkMotion2_Anim_inline(gobj: *mut Item_GObj) {
    let item = get_item(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as _);
    Item_SetEffectHitlagCallbacks(item);
}

pub unsafe extern "C" fn itKireihana_UnkMotion2_Anim(gobj: *mut Item_GObj) -> i32 {
    let item = get_item(gobj);
    ext::it_8027A160((*(*item).xBBC_dynamicBoneTable).bones[1], item);
    if ext::it_80272C6C(gobj) == 0 {
        let old = (*item).xDD4_itemVar.kireihana.timer_unk0;
        (*item).xDD4_itemVar.kireihana.timer_unk0 = old.wrapping_sub(1);
        if old <= 0 {
            it_802D0774(gobj);
        } else {
            itKireihana_UnkMotion2_Anim_inline(gobj);
        }
    }
    0
}

pub unsafe extern "C" fn itKireihana_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    let item = get_item(gobj);
    (*item).xDD4_itemVar.kireihana.timer_unk1 = (*item).xDD4_itemVar.kireihana.timer_unk1.wrapping_sub(1);
    if (*item).xDD4_itemVar.kireihana.timer_unk1 == 0 {
        let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itkireihanaAttributes;
        ext::it_8027ADEC(0x470, gobj, (*(*item).xBBC_dynamicBoneTable).bones[2], 2.6);
        (*item).xDD4_itemVar.kireihana.timer_unk1 = (*attrs).x8 as i32;
    }
    ext::it_8027A344(gobj);
    (*item).xDD4_itemVar.kireihana.fall_speed += (*(*item).xCC_item_attr).x10_fall_speed;
    if (*item).xDD4_itemVar.kireihana.fall_speed > (*(*item).xCC_item_attr).x14_fall_speed_max {
        (*item).xDD4_itemVar.kireihana.fall_speed = (*(*item).xCC_item_attr).x14_fall_speed_max;
    }
    (*item).x40_vel.y = -(*item).xDD4_itemVar.kireihana.fall_speed;
}

pub unsafe extern "C" fn itKireihana_UnkMotion2_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8026E248(gobj, Some(it_802D01A0));
    0
}

pub unsafe extern "C" fn it_802D05D8(gobj: *mut HSD_GObj) {
    let item = get_item(gobj);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE as _);
    Item_SetEffectHitlagCallbacks(item);
    (*item).xDD4_itemVar.kireihana.fall_speed = 0.0;
}

pub unsafe extern "C" fn it_802D062C(gobj: *mut Item_GObj) {
    let item = get_item(gobj);
    ext::Item_80268E5C(gobj, 3, (ITEM_UNK_0x1 | ITEM_HIT_PRESERVE) as _);
    Item_SetEffectHitlagCallbacks(item);
    ext::it_8027A160((*(*item).xBBC_dynamicBoneTable).bones[1], item);
    (*item).xDD4_itemVar.kireihana.fall_speed = 0.0;
}

pub unsafe extern "C" fn itKireihana_UnkMotion3_Anim(gobj: *mut Item_GObj) -> i32 {
    let item = get_item(gobj);
    (*item).xDD4_itemVar.kireihana.x70 = (*item).xDD4_itemVar.kireihana.x8;
    ext::it_8027A160((*(*item).xBBC_dynamicBoneTable).bones[1], item);
    if ext::it_80272C6C(gobj) == 0 {
        return 1;
    }
    0
}

pub unsafe extern "C" fn itKireihana_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    ext::it_8027A344(gobj);
}

pub unsafe extern "C" fn itKireihana_UnkMotion3_Coll(gobj: *mut Item_GObj) -> i32 {
    let item = get_item(gobj);
    if ext::it_8026DA08(gobj) == 0 && (*item).xDD4_itemVar.kireihana.x70 < 0.0 && (*item).xDD4_itemVar.kireihana.x8 > 0.0 {
        it_802D07C0(gobj);
    }
    0
}

pub unsafe extern "C" fn it_802D0774(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE as _);
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn it_802D07C0(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    ext::Item_80268E5C(gobj, 4, (ITEM_UNK_0x1 | ITEM_HIT_PRESERVE) as _);
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn itKireihana_UnkMotion4_Anim(gobj: *mut Item_GObj) -> i32 {
    let item = get_item(gobj);
    ext::it_8027A160((*(*item).xBBC_dynamicBoneTable).bones[1], item);
    if ext::it_80272C6C(gobj) == 0 {
        return 1;
    }
    0
}

pub unsafe extern "C" fn itKireihana_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    let item = get_item(gobj);
    ext::it_8027A344(gobj);
    (*item).xDD4_itemVar.kireihana.fall_speed += (*(*item).xCC_item_attr).x10_fall_speed;
    if (*item).xDD4_itemVar.kireihana.fall_speed > (*(*item).xCC_item_attr).x14_fall_speed_max {
        (*item).xDD4_itemVar.kireihana.fall_speed = (*(*item).xCC_item_attr).x14_fall_speed_max;
    }
    (*item).x40_vel.y = -(*item).xDD4_itemVar.kireihana.fall_speed;
}

pub unsafe extern "C" fn itKireihana_UnkMotion4_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8026E248(gobj, Some(it_802D062C));
    0
}

pub unsafe extern "C" fn it_802D08F0(gobj: *mut Item_GObj) {
    Item_EnterAirStateWithHitlagAndStateDesc(gobj);
}

pub unsafe extern "C" fn itKireihana_UnkMotion0_Anim(gobj: *mut Item_GObj) -> i32 {
    ext::it_80279FF8(gobj);
    0
}

pub unsafe extern "C" fn itKireihana_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    ext::it_8027A09C(gobj);
}

pub unsafe extern "C" fn itKireihana_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8027A118(gobj, Some(it_802D0138))
}
