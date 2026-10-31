//! Port of src/melee/it/kinds/itmew.c: Mew (Pokeball Pokemon, logic table 22). Callees of unported TUs go
//! through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itmew.rs.
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
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn Stage_GetBlastZoneTopOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneTopOffset() };
    fn efLib_PauseAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_PauseAll(a0) };
    fn efLib_ResumeAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_ResumeAll(a0) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
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

/// `ItemStateTable it_803F80C8[]`
pub static mut it_803F80C8: [ItemStateTable; 3] = [
    st_row(0, pred_b(itMew_UnkMotion0_Anim), Some(itMew_UnkMotion0_Phys), pred_b(itMew_UnkMotion0_Coll)),
    st_row(1, pred_b(itMew_UnkMotion1_Anim), Some(itMew_UnkMotion1_Phys), pred_b(itMew_UnkMotion1_Coll)),
    st_row(2, pred_b(itMew_UnkMotion2_Anim), Some(itMew_UnkMotion2_Phys), pred_b(itMew_UnkMotion2_Coll)),
];

/// `Item_SetEffectHitlagCallbacks` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_SetEffectHitlagCallbacks(ip: *mut Item) {
    (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}

/// `Item_EnterAirStateWithHitlagAndStateDesc` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_EnterAirStateWithHitlagAndStateDesc(gobj: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    ext::it_802762BC(item);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    (*item).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*item).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
    ext::it_80273670(gobj, 0, 0.0);
}

pub unsafe extern "C" fn itMew_Logic22_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itMew_Logic22_Spawned");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut MewVars;

    (*ip).facing_dir = 0.0;
    it_802D3D94(gobj);
    ext::it_80279CDC(gobj, (*attr).x0);
}

pub unsafe extern "C" fn it_802D3B6C(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D3B6C");
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe extern "C" fn it_802D3B8C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D3B8C");
    let ip = GET_ITEM(gobj);

    ext::it_80273454(gobj);
    it_802D3BE0(gobj);
    ext::Item_8026AE84(ip, 0x2729, 0x7F, 0x40);
}

pub unsafe extern "C" fn it_802D3BE0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D3BE0");
    let ip = GET_ITEM(gobj);
    let jobj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;

    ext::it_8027ADEC(0x46B, gobj, jobj, 2.8);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn itMew_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMew_UnkMotion1_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        it_802D3C9C(gobj);
    }
    false
}

pub unsafe extern "C" fn itMew_UnkMotion1_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itMew_UnkMotion1_Phys");}

pub unsafe extern "C" fn itMew_UnkMotion1_Coll(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMew_UnkMotion1_Coll");
    false
}

pub unsafe extern "C" fn it_802D3C9C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D3C9C");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut MewVars;

    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
    Item_SetEffectHitlagCallbacks(ip);
    if ext::HSD_Randi(2) != 0 {
        (*ip).x40_vel.x = (*attr).x4;
    } else {
        (*ip).x40_vel.x = -(*attr).x4;
    }
    (*ip).x40_vel.y = (*attr).x8;
}

pub unsafe extern "C" fn itMew_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMew_UnkMotion2_Anim");
    let ip = GET_ITEM(gobj);

    if (*ip).pos.y > ext::Stage_GetBlastZoneTopOffset() {
        return true;
    }
    false
}

pub unsafe extern "C" fn itMew_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itMew_UnkMotion2_Phys");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut MewVars;
    (*ip).x40_vel.y += (*attr).xC;
}

pub unsafe extern "C" fn itMew_UnkMotion2_Coll(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMew_UnkMotion2_Coll");
    false
}

pub unsafe extern "C" fn it_802D3D94(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D3D94");
    Item_EnterAirStateWithHitlagAndStateDesc(gobj);
}

pub unsafe extern "C" fn itMew_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMew_UnkMotion0_Anim");
    ext::it_80279FF8(gobj);
    false
}

pub unsafe extern "C" fn itMew_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itMew_UnkMotion0_Phys");
    if ext::it_8027A09C(gobj) != 0 {
        let ip_arg = GET_ITEM(gobj);
        ext::it_80273454(gobj);
        let ip = GET_ITEM(gobj);
        ext::it_8027ADEC(0x46B, gobj, HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, 2.8);
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
        Item_SetEffectHitlagCallbacks(ip);
        ext::Item_8026AE84(ip_arg, 0x2729, 0x7F, 0x40);
    }
}

pub unsafe extern "C" fn itMew_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMew_UnkMotion0_Coll");
    ext::it_8027A118(gobj, Some(it_802D3B8C)) != 0
}
