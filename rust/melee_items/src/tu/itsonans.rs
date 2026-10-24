//! Port of src/melee/it/kinds/itsonans.c: Sonans / Wobbuffet (logic table 9). Callees of unported TUs go
//! through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itsonans.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_mut, unused_variables, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

use super::kinds_inl::*;

// // BEGIN GENERATED EXT (rust/melee_items/gen/kinds/gen_tu.py; the callees of the C TU)
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
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efLib_PauseAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::eflib::efLib_PauseAll(a0) };
    fn efLib_ResumeAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::eflib::efLib_ResumeAll(a0) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_80272460(a0: *mut HitCapsule, a1: u32, a2: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_80272460(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80275444(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275444(&mut *((*a0).user_data as *mut Item)) };
    fn it_802754BC(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802754bc(&mut *((*a0).user_data as *mut Item)) };
    fn it_802754D4(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802754d4(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756E0(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_802756e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_80279FF8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279FF8(a0) };
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027A09C(a0) as _ };
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::it_279C::it_8027A118(a0, a1) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn efLib_PauseAll(a0: *mut HSD_GObj) { super::efLib_PauseAll(a0) }
        pub unsafe extern "C" fn efLib_ResumeAll(a0: *mut HSD_GObj) { super::efLib_ResumeAll(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

#[inline]
fn ABS(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `Item_SetEffectHitlagCallbacks` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_SetEffectHitlagCallbacks(ip: *mut Item) {
    (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}

/// `Item_EnterAirStateWithHitlag` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_EnterAirStateWithHitlag(gobj: *mut Item_GObj, msid: i32) {
    let item = GET_ITEM(gobj);
    ext::it_802762BC(item);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE);
    (*item).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*item).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}

/// `ItemStateTable it_803F7CA0[]`
pub static mut it_803F7CA0: [ItemStateTable; 3] = [
    st_row(0, pred_b(itSonans_UnkMotion0_Anim), Some(itSonans_UnkMotion0_Phys), pred_b(itSonans_UnkMotion0_Coll)),
    st_row(1, pred_b(itSonans_UnkMotion1_Anim), Some(itSonans_UnkMotion1_Phys), pred_b(itSonans_UnkMotion1_Coll)),
    st_row(-1, pred_b(itSonans_UnkMotion2_Anim), Some(itSonans_UnkMotion2_Phys), pred_b(itSonans_UnkMotion2_Coll)),
];

pub unsafe extern "C" fn it_802CD44C(gobj: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itsonansAttributes;
    (*item).facing_dir = 0.0;
    (*item).xDAC_itcmd_var0 = 0;
    (*item).xD44_lifeTimer = (*attrs).x24 as f32;
    (*item).xDD4_itemVar.sonans.x60 = 0.0;
    (*item).xDD4_itemVar.sonans.x64 = 0.0;
    (*item).xDD4_itemVar.sonans.x68 = 0.0;
    ext::it_80279CDC(gobj, (*attrs).x0);
    ext::it_80275444(gobj);
    it_802CDAA8(gobj);
}

pub unsafe extern "C" fn it_802CD4D8(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn it_802CD4DC(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe extern "C" fn it_802CD4FC(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itsonansAttributes;
    let mut angle: f32;
    let max_angle: f32;
    if (*ip).ground_or_air == GA_Air {
        ext::it_80272860(gobj, (*(*ip).xCC_item_attr).x10_fall_speed, (*(*ip).xCC_item_attr).x14_fall_speed_max);
    }
    if (*ip).xDAC_itcmd_var0 != 0 {
        angle = (*ip).xDD4_itemVar.sonans.x64;
        if angle > 0.0 && (*ip).xDD4_itemVar.sonans.x60 > 0.0 {
            (*ip).xDD4_itemVar.sonans.x60 += (*attrs).x14;
        } else if angle <= 0.0 && (*ip).xDD4_itemVar.sonans.x60 <= 0.0 {
            (*ip).xDD4_itemVar.sonans.x60 -= (*attrs).x14;
        } else if angle > 0.0 && (*ip).xDD4_itemVar.sonans.x60 <= 0.0 {
            (*ip).xDD4_itemVar.sonans.x60 -= (*attrs).x10;
        } else if angle <= 0.0 && (*ip).xDD4_itemVar.sonans.x60 > 0.0 {
            (*ip).xDD4_itemVar.sonans.x60 += (*attrs).x10;
        }
        angle = ABS((*ip).xDD4_itemVar.sonans.x64);
        if angle < (*attrs).x1C {
            angle = ABS((*ip).xDD4_itemVar.sonans.x60);
            if angle < (*attrs).x1C {
                (*ip).xDD4_itemVar.sonans.x64 = 0.0;
                (*ip).xDD4_itemVar.sonans.x60 = 0.0;
                (*ip).xDAC_itcmd_var0 = 0;
            } else {
                (*ip).xDD4_itemVar.sonans.x64 += (*ip).xDD4_itemVar.sonans.x60;
            }
        } else {
            (*ip).xDD4_itemVar.sonans.x64 += (*ip).xDD4_itemVar.sonans.x60;
        }
        angle = (*ip).xDD4_itemVar.sonans.x64;
        max_angle = (*attrs).x18;
        if angle > max_angle {
            (*ip).xDD4_itemVar.sonans.x60 = ((*ip).xDD4_itemVar.sonans.x60 as f64 * -0.9) as f32;
            (*ip).xDD4_itemVar.sonans.x64 = (*attrs).x18;
        } else if angle < -max_angle {
            (*ip).xDD4_itemVar.sonans.x60 = ((*ip).xDD4_itemVar.sonans.x60 as f64 * -0.9) as f32;
            (*ip).xDD4_itemVar.sonans.x64 = -(*attrs).x18;
        }
        HSD_JObjSetRotationZ((*(*ip).xBBC_dynamicBoneTable).bones[4], 0.017453292_f32 * (*ip).xDD4_itemVar.sonans.x64);
    }
    ext::it_80272460(&mut (*ip).x5D4_hitboxes[0].hit, (*ip).xDD4_itemVar.sonans.x68 as u32, gobj);
    (*ip).xDD4_itemVar.sonans.x68 -= (*attrs).x20;
}

pub unsafe extern "C" fn itSonans_Logic9_DmgDealt(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.sonans.x60 *= -1.0;
    false
}

pub unsafe extern "C" fn it_802CD7D4(gobj: *mut Item_GObj) -> bool {
    let item = GET_ITEM(gobj);
    let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itsonansAttributes;
    if (*item).xDAC_itcmd_var0 == 0 {
        (*item).xDD4_itemVar.sonans.x60 = (*attrs).x8 * ((*item).xCA0 as f32 * (*item).xCCC_incDamageDirection);
        (*item).xDAC_itcmd_var0 = 1;
    } else if ABS((*item).xDD4_itemVar.sonans.x60) < ABS((*attrs).x8 * ((*item).xCA0 as f32 * (*item).xCCC_incDamageDirection)) {
        (*item).xDD4_itemVar.sonans.x60 = (*attrs).x8 * ((*item).xCA0 as f32 * (*item).xCCC_incDamageDirection);
    }
    if (*item).xCA0 as f32 * (*attrs).x4 > (*attrs).xC {
        (*item).xDD4_itemVar.sonans.x68 = (*attrs).xC;
    } else {
        (*item).xDD4_itemVar.sonans.x68 = (*item).xCA0 as f32 * (*attrs).x4;
    }
    false
}

pub unsafe extern "C" fn itSonans_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    let item = GET_ITEM(gobj);
    if ext::it_80272C6C(gobj) == 0 {
        it_802CD9C0(gobj);
        ext::Item_8026AE84(item, 0x272B, 0x7F, 0x40);
    }
    (*item).xD44_lifeTimer -= 1.0;
    false
}

pub unsafe extern "C" fn itSonans_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    it_802CD4FC(gobj);
}

pub unsafe extern "C" fn itSonans_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    if (*GET_ITEM(gobj)).ground_or_air == GA_Air {
        ext::it_8026E15C(gobj, Some(it_802CD4D8));
    } else {
        ext::it_8026D62C(gobj, Some(it_802CD4D8));
    }
    false
}

pub unsafe extern "C" fn it_802CD9C0(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn itSonans_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let zero: i32 = 0;

    let lifetime = (*ip).xD44_lifeTimer - 1.0;
    (*ip).xD44_lifeTimer = lifetime;

    lifetime == zero as f32
}

pub unsafe extern "C" fn itSonans_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    it_802CD4FC(gobj);
}

pub unsafe extern "C" fn itSonans_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    if (*GET_ITEM(gobj)).ground_or_air == GA_Air {
        ext::it_8026E15C(gobj, Some(it_802CD4D8));
    } else {
        ext::it_8026D62C(gobj, Some(it_802CD4D8));
    }
    false
}

pub unsafe extern "C" fn it_802CDAA8(gobj: *mut Item_GObj) {
    Item_EnterAirStateWithHitlag(gobj, 2);
}

pub unsafe extern "C" fn itSonans_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    ext::it_80279FF8(gobj);
    false
}

pub unsafe extern "C" fn itSonans_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    if ext::it_8027A09C(gobj) != 0 {
        ext::it_80273454(gobj);
        ext::it_802756E0(gobj);
        ext::it_802754D4(gobj);
        ext::it_802754BC(gobj);
        let ip = GET_ITEM(gobj);
        ext::Item_80268E5C(gobj, 0, 2);
        Item_SetEffectHitlagCallbacks(ip);
    }
}

pub unsafe extern "C" fn itSonans_UnkMotion2_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8027A118(gobj, Some(it_802CD4D8)) != 0
}
