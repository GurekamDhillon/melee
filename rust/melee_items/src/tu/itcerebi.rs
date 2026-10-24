//! Port of src/melee/it/kinds/itcerebi.c: Celebi (Pokeball Pokemon, logic table 23). Callees of unported
//! TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction)
//! by tests/kinds_itcerebi.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

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
    fn HSD_Randi(a0: i32) -> i32;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Stage_GetBlastZoneTopOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneTopOffset() };
    fn efLib_PauseAll(a0: *mut HSD_GObj);
    fn efLib_ResumeAll(a0: *mut HSD_GObj);
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj, a3: *mut f32) -> *mut c_void;
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80273670(a0: *mut HSD_GObj, a1: i32, a2: f32) => unsafe { crate::tu::it_2725::it_80273670(a0, a1, a2) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_80279FF8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279FF8(a0) };
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027A09C(a0) as _ };
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::it_279C::it_8027A118(a0, a1) as _ };
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

/// `ItemStateTable it_803F80F8[]`
pub static mut it_803F80F8: [ItemStateTable; 3] = [
    st_row(0, pred_b(itCerebi_UnkMotion0_Anim), Some(itCerebi_UnkMotion0_Phys), pred_b(itCerebi_UnkMotion0_Coll)),
    st_row(1, pred_b(itCerebi_UnkMotion1_Anim), Some(itCerebi_UnkMotion1_Phys), pred_b(itCerebi_UnkMotion1_Coll)),
    st_row(2, pred_b(itCerebi_UnkMotion2_Anim), Some(itCerebi_UnkMotion2_Phys), pred_b(itCerebi_UnkMotion2_Coll)),
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

pub unsafe extern "C" fn itCerebi_Logic23_Spawned(gobj: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut f32;
    (*item).facing_dir = 0.0;
    it_802D4168(gobj);
    ext::it_80279CDC(gobj, *attr);
}

/// 2D3F4C
pub unsafe extern "C" fn it_802D3F4C(gobj: *mut Item_GObj, arg1: *mut Item_GObj) {
    ext::it_8026B894(gobj, arg1);
}

/// 2D3F6C
pub unsafe extern "C" fn it_802D3F6C(gobj: *mut Item_GObj) {
    ext::it_80273454(gobj);
    it_802D3FA0(gobj);
}

/// 2D3FA0
pub unsafe extern "C" fn it_802D3FA0(gobj: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let mut f: f32 = 1.0;
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    Item_SetEffectHitlagCallbacks(item);
    ext::efSync_Spawn(0x472, gobj, jobj, &mut f);
}

pub unsafe extern "C" fn itCerebi_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        it_802D4070(gobj);
    }
    false
}

pub unsafe extern "C" fn itCerebi_UnkMotion1_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itCerebi_UnkMotion1_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

/// 2D4070
pub unsafe extern "C" fn it_802D4070(gobj: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut f32;

    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
    Item_SetEffectHitlagCallbacks(item);
    if ext::HSD_Randi(2) != 0 {
        (*item).x40_vel.x = *attrs.add(1);
    } else {
        (*item).x40_vel.x = -*attrs.add(1);
    }
    (*item).x40_vel.y = *attrs.add(2);
}

pub unsafe extern "C" fn itCerebi_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    let item = GET_ITEM(gobj);
    if (*item).pos.y > ext::Stage_GetBlastZoneTopOffset() {
        return true;
    }
    false
}

pub unsafe extern "C" fn itCerebi_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    (*item).x40_vel.y += *((*(*item).xC4_article_data).x4_specialAttributes as *mut f32).add(3);
}

pub unsafe extern "C" fn itCerebi_UnkMotion2_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

/// 2D4168
pub unsafe extern "C" fn it_802D4168(gobj: *mut Item_GObj) {
    Item_EnterAirStateWithHitlagAndStateDesc(gobj);
}

pub unsafe extern "C" fn itCerebi_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    ext::it_80279FF8(gobj);
    false
}

pub unsafe extern "C" fn itCerebi_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    if ext::it_8027A09C(gobj) != 0 {
        ext::it_80273454(gobj);
        let item = GET_ITEM(gobj);
        let jobj = GET_JOBJ(gobj);
        let mut f: f32 = 1.0;
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
        Item_SetEffectHitlagCallbacks(item);
        ext::efSync_Spawn(0x472, gobj, jobj, &mut f);
    }
}

pub unsafe extern "C" fn itCerebi_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8027A118(gobj, Some(it_802D3F6C)) != 0
}
