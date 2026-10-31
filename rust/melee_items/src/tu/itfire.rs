//! Native port of src/melee/it/kinds/itfire.c: the Fire item (Mr. Game & Watch-style flame
//! pickup): its state callbacks and the state table `it_803F7C10`. Callees in other TUs go
//! through `ext` (hooks; unset panics "TODO"). Checked against the C (the whole TU, shipping
//! contraction) by tests/tu_itfire.rs.
//!
//! Header inlines compiled into the TU: `Item_SetEffectHitlagCallbacks`,
//! `Item_EnterAirStateWithHitlagAndStateDesc`, `GET_ITEM`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

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
    fn it_8026E7E0(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E7E0(a0, a1) };
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
unsafe fn get_item(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `Item_SetEffectHitlagCallbacks`.
#[inline]
unsafe fn Item_SetEffectHitlagCallbacks(ip: *mut Item) {
    (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}

/// `Item_EnterAirStateWithHitlagAndStateDesc`.
#[inline]
unsafe fn Item_EnterAirStateWithHitlagAndStateDesc(gobj: *mut Item_GObj) {
    let item = get_item(gobj);
    ext::it_802762BC(item);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as _);
    Item_SetEffectHitlagCallbacks(item);
    ext::it_80273670(gobj, 0, 0.0);
}

#[inline]
unsafe fn attrs(ip: *mut Item) -> *mut itFireAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itFireAttributes
}

pub static mut it_803F7C10: [ItemStateTable; 3] = [
    ItemStateTable { anim_id: 0, animated: Some(itFire_UnkMotion0_Anim), physics_updated: Some(itFire_UnkMotion0_Phys), collided: Some(itFire_UnkMotion0_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itFire_UnkMotion1_Anim), physics_updated: Some(itFire_UnkMotion1_Phys), collided: Some(itFire_UnkMotion1_Coll) },
    ItemStateTable { anim_id: 2, animated: Some(itFire_UnkMotion2_Anim), physics_updated: Some(itFire_UnkMotion2_Phys), collided: Some(itFire_UnkMotion2_Coll) },
];

pub unsafe extern "C" fn itFire_Logic6_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itFire_Logic6_Spawned");
    let ip = get_item(gobj);
    let a = attrs(ip);
    (*ip).facing_dir = 0.0;
    (*ip).xDAC_itcmd_var0 = 0;
    it_802CC944(gobj);
    ext::it_80279CDC(gobj, (*a).x0);
}

pub unsafe extern "C" fn it_802CC740(_gobj: *mut Item_GObj) { melee_core::ct!("it_802CC740");}

pub unsafe extern "C" fn itFire_Logic6_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itFire_Logic6_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe extern "C" fn itFire_UnkMotion1_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFire_UnkMotion1_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        it_802CC7D8(gobj);
    }
    0
}

pub unsafe extern "C" fn itFire_UnkMotion1_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itFire_UnkMotion1_Phys");}

pub unsafe extern "C" fn itFire_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFire_UnkMotion1_Coll");
    ext::it_8026E7E0(gobj, Some(it_802CC7D4));
    0
}

pub unsafe extern "C" fn it_802CC7D4(_gobj: *mut Item_GObj) { melee_core::ct!("it_802CC7D4");}

pub unsafe extern "C" fn it_802CC7D8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CC7D8");
    let ip = get_item(gobj);
    let a = attrs(ip);
    ext::it_8027ADEC(0x463, gobj, (*gobj).hsd_obj as *mut HSD_JObj, 1.1);
    (*ip).x40_vel.y = (*a).x8;
    ext::Item_80268E5C(gobj, 2, 2);
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn itFire_UnkMotion2_Anim_inline(gobj: *mut Item_GObj) {
    melee_core::ct!("itFire_UnkMotion2_Anim_inline");
    let ip = get_item(gobj);
    let a = attrs(ip);
    ext::it_8027ADEC(0x463, gobj, (*gobj).hsd_obj as *mut HSD_JObj, 1.1);
    (*ip).x40_vel.y = (*a).x8;
    ext::Item_80268E5C(gobj, 2, 2);
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn itFire_UnkMotion2_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFire_UnkMotion2_Anim");
    let ip = get_item(gobj);
    if ext::it_80272C6C(gobj) == 0 {
        itFire_UnkMotion2_Anim_inline(gobj);
    }
    if (*ip).pos.y > ext::Stage_GetBlastZoneTopOffset() {
        1
    } else {
        0
    }
}

pub unsafe extern "C" fn itFire_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itFire_UnkMotion2_Phys");
    let ip = get_item(gobj);
    let a = attrs(ip);
    (*ip).x40_vel.y += (*a).xC;
}

pub unsafe extern "C" fn itFire_UnkMotion2_Coll(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFire_UnkMotion2_Coll");
    0
}

pub unsafe extern "C" fn it_802CC944(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CC944");
    Item_EnterAirStateWithHitlagAndStateDesc(gobj);
}

pub unsafe extern "C" fn itFire_UnkMotion0_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFire_UnkMotion0_Anim");
    ext::it_80279FF8(gobj);
    0
}

pub unsafe extern "C" fn itFire_UnkMotion0_Phys_inline(gobj: *mut Item_GObj) {
    melee_core::ct!("itFire_UnkMotion0_Phys_inline");
    let ip = get_item(gobj);
    ext::Item_80268E5C(gobj, 1, 2);
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn itFire_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itFire_UnkMotion0_Phys");
    if ext::it_8027A09C(gobj) != 0 {
        let ip = get_item(gobj);
        let a = attrs(ip);
        ext::it_8027ADEC(0x464, gobj, (*gobj).hsd_obj as *mut HSD_JObj, 1.1);
        ext::it_80273454(gobj);
        (*ip).x40_vel.y = (*a).x4;
        itFire_UnkMotion0_Phys_inline(gobj);
        (*ip).xDD1_flag.anon_0.set_b1(1);
    }
}

pub unsafe extern "C" fn itFire_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFire_UnkMotion0_Coll");
    ext::it_8027A118(gobj, Some(it_802CC740))
}
