//! Native port of src/melee/it/kinds/itfushigibana.c: the Venusaur (Fushigibana) stage-hazard item: the
//! razor-leaf sprite; state 0 waits to land, state 1 counts a lifetime and emits effects on a timer, state 2
//! falls. Callees in other TUs go through `ext` (unported: panics "TODO"); the header inlines the TU uses
//! (`Item_SetEffectHitlagCallbacks`, `Item_EnterStateWithEffectHitlag`) are ported here. Checked against
//! the C (the whole TU, shipping contraction) by tests/tu_itfushigibana.rs.
//!
//! No fused multiply-add sites.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables)]

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
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8);
    fn efLib_PauseAll(a0: *mut HSD_GObj);
    fn efLib_ResumeAll(a0: *mut HSD_GObj);
    fn it_8026BDB4(a0: *mut HSD_GObj);
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32);
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32;
    fn it_80273454(a0: *mut HSD_GObj);
    fn it_80273670(a0: *mut HSD_GObj, a1: i32, a2: f32);
    fn it_80275158(a0: *mut HSD_GObj, a1: f32);
    fn it_80275288(a0: *mut HSD_GObj, a1: i32, a2: i32);
    fn it_802762BC(a0: *mut Item);
    fn it_80276CB8(a0: *mut HSD_GObj);
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32);
    fn it_80279FF8(a0: *mut HSD_GObj);
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32;
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32;
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
    (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}

/// `Item_EnterStateWithEffectHitlag` (it/inlines.h).
#[inline]
pub unsafe fn Item_EnterStateWithEffectHitlag(gobj: *mut Item_GObj, msid: i32) {
    let ip = item_of(gobj);
    ext::it_802762BC(ip);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE);
    (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}

/// `ItemStateTable it_803F82F0[3]`.
pub static it_803F82F0: [ItemStateTable; 3] = [
    ItemStateTable { anim_id: 0, animated: Some(itFushigibana_UnkMotion0_Anim), physics_updated: Some(itFushigibana_UnkMotion0_Phys), collided: Some(itFushigibana_UnkMotion0_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itFushigibana_UnkMotion1_Anim), physics_updated: Some(itFushigibana_UnkMotion1_Phys), collided: Some(itFushigibana_UnkMotion1_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itFushigibana_UnkMotion2_Anim), physics_updated: Some(itFushigibana_UnkMotion2_Phys), collided: Some(itFushigibana_UnkMotion2_Coll) },
];

/// `itFushigibana_UnkMotion1_Anim_inline1` (static): enters state 1.
pub unsafe fn itFushigibana_UnkMotion1_Anim_inline1(gobj: *mut Item_GObj) {
    let item = item_of(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    (*item).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*item).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
    (*item).xDD4_itemVar.fushigibana.x60.anon_0.set_b0(1);
}

/// `itFushigibana_UnkMotion1_Anim_inline2` (static): the lifetime countdown.
pub unsafe fn itFushigibana_UnkMotion1_Anim_inline2(gobj: *mut Item_GObj) -> i32 {
    let item = item_of(gobj);
    (*item).xD44_lifeTimer -= 1.0;
    if (*item).xD44_lifeTimer <= 0.0 {
        1
    } else {
        0
    }
}

/// `void itFushigibana_Logic29_Spawned(Item_GObj*)`: init from the attributes.
pub unsafe extern "C" fn itFushigibana_Logic29_Spawned(gobj: *mut Item_GObj) {
    let item = item_of(gobj);
    let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itFushigibanaAttributes;
    ext::it_80279CDC(gobj, (*attrs).x0);
    ext::Item_8026AE84(item, 0x272f, 0x7f, 0x40);
    ext::it_80275158(gobj, (*attrs).x4);
    (*item).xDD4_itemVar.fushigibana.x60.anon_0.set_b0(0);
    (*item).xDD4_itemVar.fushigibana.x64 = (*attrs).x8;
    it_802D70A4(gobj);
}

/// `void it_802D705C(Item_GObj*)`: landed: clears a flag and enters state 1.
pub unsafe extern "C" fn it_802D705C(gobj: *mut Item_GObj) {
    let ip = item_of(gobj);
    (*ip).xDCE_flag.anon_0.set_b7(0);
    ext::it_80273454(gobj);
    it_802D718C(gobj);
}

/// `void it_802D70A4(Item_GObj*)`: enters state 0 with the hitlag callbacks.
pub unsafe extern "C" fn it_802D70A4(gobj: *mut Item_GObj) {
    Item_EnterStateWithEffectHitlag(gobj, 0);
    ext::it_80273670(gobj, 0, 0.0);
    ext::it_8026BDB4(gobj);
}

/// `bool itFushigibana_UnkMotion0_Anim(Item_GObj*)`.
pub unsafe extern "C" fn itFushigibana_UnkMotion0_Anim(gobj: *mut Item_GObj) -> i32 {
    ext::it_80279FF8(gobj);
    0
}

/// `void itFushigibana_UnkMotion0_Phys(Item_GObj*)`.
pub unsafe extern "C" fn itFushigibana_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    ext::it_8027A09C(gobj);
}

/// `bool itFushigibana_UnkMotion0_Coll(Item_GObj*)`.
pub unsafe extern "C" fn itFushigibana_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8027A118(gobj, Some(it_802D705C))
}

/// `void it_802D718C(Item_GObj*)`: enters state 1.
pub unsafe extern "C" fn it_802D718C(gobj: *mut Item_GObj) {
    let item = item_of(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    Item_SetEffectHitlagCallbacks(item);
    (*item).xDD4_itemVar.fushigibana.x60.anon_0.set_b0(1);
}

/// `bool itFushigibana_UnkMotion1_Anim(Item_GObj*)`: emits the periodic effect, counts the lifetime down.
pub unsafe extern "C" fn itFushigibana_UnkMotion1_Anim(gobj: *mut Item_GObj) -> i32 {
    let item = item_of(gobj);
    let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itFushigibanaAttributes;
    if ext::it_80272C6C(gobj) == 0 {
        itFushigibana_UnkMotion1_Anim_inline1(gobj);
    }
    if (*item).xDD4_itemVar.fushigibana.x64 == 0.0 {
        (*item).xDD4_itemVar.fushigibana.x64 = (*attrs).x8;
        ext::it_80275288(gobj, 4, 0x2730);
    }
    (*item).xDD4_itemVar.fushigibana.x64 -= 1.0;
    itFushigibana_UnkMotion1_Anim_inline2(gobj)
}

/// `void itFushigibana_UnkMotion1_Phys(Item_GObj*)`.
pub unsafe extern "C" fn itFushigibana_UnkMotion1_Phys(gobj: *mut Item_GObj) {}

/// `bool itFushigibana_UnkMotion1_Coll(Item_GObj*)`.
pub unsafe extern "C" fn itFushigibana_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8026D62C(gobj, Some(it_802D7328));
    ext::it_80276CB8(gobj);
    0
}

/// `void it_802D7328(Item_GObj*)`: enters state 2 (falling).
pub unsafe extern "C" fn it_802D7328(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
    ext::it_80273670(gobj, 0, 0.0);
}

/// `bool itFushigibana_UnkMotion2_Anim(Item_GObj*)`.
pub unsafe extern "C" fn itFushigibana_UnkMotion2_Anim(gobj: *mut Item_GObj) -> i32 {
    0
}

/// `void itFushigibana_UnkMotion2_Phys(Item_GObj*)`: gravity.
pub unsafe extern "C" fn itFushigibana_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    let ip = item_of(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

/// `bool itFushigibana_UnkMotion2_Coll(Item_GObj*)`.
pub unsafe extern "C" fn itFushigibana_UnkMotion2_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8026E15C(gobj, Some(it_802D705C));
    0
}
