//! Native port of src/melee/it/kinds/itfreezer.c: the Freezer item (the ice-shooting stage-hazard
//! item): spawns effects on a timer while idle (state 0 waits, 1 emits, 2 flies up and is removed above
//! the top blast zone). Callees in other TUs go through `ext` (unported: panics "TODO"); the header
//! inlines the TU uses (`Item_SetEffectHitlagCallbacks`, `Item_EnterAirStateWithHitlagAndStateDesc`) are
//! ported here. Checked against the C (the whole TU, shipping contraction) by tests/tu_itfreezer.rs.
//!
//! No fused multiply-add sites.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables)]

use core::ffi::c_void;
use core::ptr::null_mut;
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
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Stage_GetBlastZoneTopOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneTopOffset() };
    fn efLib_PauseAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_PauseAll(a0) };
    fn efLib_ResumeAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_ResumeAll(a0) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80273670(a0: *mut HSD_GObj, a1: i32, a2: f32) => unsafe { crate::tu::it_2725::it_80273670(a0, a1, a2) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_80279FF8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279FF8(a0) };
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027A09C(a0) as _ };
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::it_279C::it_8027A118(a0, a1) as _ };
    fn it_8027ADEC(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj, a3: f32) => unsafe { crate::tu::it_279C::it_8027ADEC(a0, a1, a2, a3) };
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
unsafe fn item_of(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `Item_SetEffectHitlagCallbacks` (it/inlines.h): pause / resume the effects around hitlag.
#[inline]
pub unsafe fn Item_SetEffectHitlagCallbacks(ip: *mut Item) {
    melee_core::ct!("Item_SetEffectHitlagCallbacks");
    (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}

/// `Item_EnterAirStateWithHitlagAndStateDesc` (itkinds inlines.h).
#[inline]
pub unsafe fn Item_EnterAirStateWithHitlagAndStateDesc(gobj: *mut Item_GObj) {
    melee_core::ct!("Item_EnterAirStateWithHitlagAndStateDesc");
    let item = item_of(gobj);
    ext::it_802762BC(item);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    (*item).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*item).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
    ext::it_80273670(gobj, 0, 0.0);
}

/// `ItemStateTable it_803F7C70[3]`.
pub static it_803F7C70: [ItemStateTable; 3] = [
    ItemStateTable { anim_id: 0, animated: Some(itFreezer_UnkMotion0_Anim), physics_updated: Some(itFreezer_UnkMotion0_Phys), collided: Some(itFreezer_UnkMotion0_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itFreezer_UnkMotion1_Anim), physics_updated: Some(itFreezer_UnkMotion1_Phys), collided: Some(itFreezer_UnkMotion1_Coll) },
    ItemStateTable { anim_id: 2, animated: Some(itFreezer_UnkMotion2_Anim), physics_updated: Some(itFreezer_UnkMotion2_Phys), collided: Some(itFreezer_UnkMotion2_Coll) },
];

/// `void itFreezer_802CCF9C(Item_GObj*)`: item init.
pub unsafe extern "C" fn itFreezer_802CCF9C(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreezer_802CCF9C");
    let item = item_of(gobj);
    let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itFreezerAttributes;
    (*item).facing_dir = 0.0;
    (*item).xDAC_itcmd_var0 = 0;
    (*item).xDB0_itcmd_var1 = 0;
    itFreezer_802CD2EC(gobj);
    ext::it_80279CDC(gobj, (*attrs).x0);
}

/// `void itFreezer_802CCFFC(Item_GObj*)`: an empty callback.
pub unsafe extern "C" fn itFreezer_802CCFFC(gobj: *mut Item_GObj) { melee_core::ct!("itFreezer_802CCFFC");}

/// `void itFreezer_802CD000(Item_GObj*, Item_GObj*)`: removes the gobj interaction references.
pub unsafe extern "C" fn itFreezer_802CD000(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itFreezer_802CD000");
    ext::it_8026B894(gobj, ref_gobj);
}

/// `bool itFreezer_UnkMotion1_Anim(Item_GObj*)`.
pub unsafe extern "C" fn itFreezer_UnkMotion1_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreezer_UnkMotion1_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        itFreezer_802CD12C(gobj);
    }
    0
}

/// `void itFreezer_UnkMotion1_Phys(Item_GObj*)`.
pub unsafe extern "C" fn itFreezer_UnkMotion1_Phys(gobj: *mut Item_GObj) { melee_core::ct!("itFreezer_UnkMotion1_Phys");}

/// `bool itFreezer_UnkMotion1_Coll(Item_GObj*)`.
pub unsafe extern "C" fn itFreezer_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreezer_UnkMotion1_Coll");
    ext::it_8026E15C(gobj, Some(itFreezer_802CCFFC));
    0
}

/// `void itFreezer_802CD090(Item_GObj*)`: accessory callback of state 1: the effect on bone 5 on request, and
/// the periodic effect on the item.
pub unsafe extern "C" fn itFreezer_802CD090(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreezer_802CD090");
    let item = item_of(gobj);
    if (*item).xDB0_itcmd_var1 != 0 {
        ext::it_8027ADEC(0x462, gobj, (*(*item).xBBC_dynamicBoneTable).bones[5], 1.4);
        (*item).xDB0_itcmd_var1 = 0;
    }
    if (*item).xDD4_itemVar.freezer.x60 <= 0 {
        let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itFreezerAttributes;
        (*item).xDD4_itemVar.freezer.x60 = (*attrs).xC;
        ext::it_8027ADEC(0x461, gobj, jobj, 1.4);
        return;
    }
    (*item).xDD4_itemVar.freezer.x60 -= 1;
}

/// `void itFreezer_802CD12C(HSD_GObj*)`: enters state 2.
pub unsafe extern "C" fn itFreezer_802CD12C(gobj: *mut HSD_GObj) {
    melee_core::ct!("itFreezer_802CD12C");
    let item = item_of(gobj);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
    Item_SetEffectHitlagCallbacks(item);
    (*item).on_accessory = Some(itFreezer_802CD290);
    (*item).xDB0_itcmd_var1 = 0;
}

/// `itFreezer_UnkMotion2_Anim_Inline` (static): same as `itFreezer_802CD12C`.
pub unsafe fn itFreezer_UnkMotion2_Anim_Inline(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreezer_UnkMotion2_Anim_Inline");
    let item = item_of(gobj);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
    Item_SetEffectHitlagCallbacks(item);
    (*item).on_accessory = Some(itFreezer_802CD290);
    (*item).xDB0_itcmd_var1 = 0;
}

/// `bool itFreezer_UnkMotion2_Anim(Item_GObj*)`: removed once above the top blast zone.
pub unsafe extern "C" fn itFreezer_UnkMotion2_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreezer_UnkMotion2_Anim");
    let item_temp = item_of(gobj);
    if ext::it_80272C6C(gobj) == 0 {
        itFreezer_UnkMotion2_Anim_Inline(gobj);
    }
    if (*item_temp).pos.y > ext::Stage_GetBlastZoneTopOffset() {
        return 1;
    }
    0
}

/// `void itFreezer_UnkMotion2_Phys(Item_GObj*)`: the launch impulse and the upward acceleration.
pub unsafe extern "C" fn itFreezer_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreezer_UnkMotion2_Phys");
    let item = item_of(gobj);
    if (*item).xDAC_itcmd_var0 != 0 {
        let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itFreezerAttributes;
        (*item).x40_vel.y = (*attrs).x4;
        (*item).xDAC_itcmd_var0 = 0;
        (*item).xDB0_itcmd_var1 = 1;
    }
    if (*item).xDB0_itcmd_var1 != 0 {
        let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itFreezerAttributes;
        (*item).x40_vel.y += (*attrs).x8;
    }
}

/// `bool itFreezer_UnkMotion2_Coll(Item_GObj*)`.
pub unsafe extern "C" fn itFreezer_UnkMotion2_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreezer_UnkMotion2_Coll");
    0
}

/// `void itFreezer_802CD290(Item_GObj*)`: accessory callback of state 2: the periodic effect.
pub unsafe extern "C" fn itFreezer_802CD290(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreezer_802CD290");
    let item = item_of(gobj);
    if (*item).xDD4_itemVar.freezer.x60 <= 0 {
        let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itFreezerAttributes;
        (*item).xDD4_itemVar.freezer.x60 = (*attrs).xC;
        ext::it_8027ADEC(0x461, gobj, jobj, 1.4);
        return;
    }
    (*item).xDD4_itemVar.freezer.x60 -= 1;
}

/// `void itFreezer_802CD2EC(Item_GObj*)`: enters the air state with hitlag callbacks.
pub unsafe extern "C" fn itFreezer_802CD2EC(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreezer_802CD2EC");
    Item_EnterAirStateWithHitlagAndStateDesc(gobj);
}

/// `bool itFreezer_UnkMotion0_Anim(Item_GObj*)`.
pub unsafe extern "C" fn itFreezer_UnkMotion0_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreezer_UnkMotion0_Anim");
    ext::it_80279FF8(gobj);
    0
}

/// `itFreezer_UnkMotion0_Phys_inline2` (static): enters state 1.
pub unsafe fn itFreezer_UnkMotion0_Phys_inline2(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreezer_UnkMotion0_Phys_inline2");
    let item = item_of(gobj);
    (*item).xDD4_itemVar.freezer.x60 = -1;
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    Item_SetEffectHitlagCallbacks(item);
    (*item).on_accessory = Some(itFreezer_802CD090);
}

/// `itFreezer_UnkMotion0_Phys_inline1` (static).
pub unsafe fn itFreezer_UnkMotion0_Phys_inline1(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreezer_UnkMotion0_Phys_inline1");
    let item = item_of(gobj);
    ext::it_80273454(gobj);
    itFreezer_UnkMotion0_Phys_inline2(gobj);
    (*item).xDD1_flag.anon_0.set_b1(1);
}

/// `void itFreezer_UnkMotion0_Phys(Item_GObj*)`.
pub unsafe extern "C" fn itFreezer_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreezer_UnkMotion0_Phys");
    if ext::it_8027A09C(gobj) != 0 {
        itFreezer_UnkMotion0_Phys_inline1(gobj);
    }
}

/// `bool itFreezer_UnkMotion0_Coll(Item_GObj*)`.
pub unsafe extern "C" fn itFreezer_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreezer_UnkMotion0_Coll");
    ext::it_8027A118(gobj, Some(itFreezer_802CCFFC))
}
