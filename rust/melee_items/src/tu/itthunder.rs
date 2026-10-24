//! Port of src/melee/it/kinds/itthunder.c: Pikachu's Thunder Pokemon (logic table 7). Callees of unported
//! TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction)
//! by tests/kinds_itthunder.rs.
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
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Stage_GetBlastZoneTopOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneTopOffset() };
    fn efLib_PauseAll(a0: *mut HSD_GObj);
    fn efLib_ResumeAll(a0: *mut HSD_GObj);
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj, a3: *mut f32) -> *mut c_void;
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
    fn it_8027A160(a0: *mut HSD_JObj, a1: *mut Item) => unsafe { crate::tu::it_279C::it_8027A160(a0, a1) };
    fn it_8027A344(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_8027a344(&mut *((*a0).user_data as *mut Item)) };
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

/// `ItemStateTable it_803F7C40[]`
pub static mut it_803F7C40: [ItemStateTable; 3] = [
    st_row(0, pred_b(itThunder_UnkMotion0_Anim), Some(itThunder_UnkMotion0_Phys), pred_b(itThunder_UnkMotion0_Coll)),
    st_row(1, pred_b(itThunder_UnkMotion1_Anim), Some(itThunder_UnkMotion1_Phys), pred_b(itThunder_UnkMotion1_Coll)),
    st_row(2, pred_b(itThunder_UnkMotion2_Anim), Some(itThunder_UnkMotion2_Phys), pred_b(itThunder_UnkMotion2_Coll)),
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

/// `itGetJObjGrandchild` (it/inlines.h, static inline)
#[inline]
unsafe fn itGetJObjGrandchild(gobj: *mut Item_GObj) -> *mut HSD_JObj {
    let jobj = GET_JOBJ(gobj);
    let mut child: *mut HSD_JObj;
    if jobj.is_null() {
        child = null_mut();
    } else {
        child = (*jobj).child;
    }
    if child.is_null() {
        child = null_mut();
    } else {
        child = (*child).child;
    }
    child
}

pub unsafe extern "C" fn itThunder_Logic7_Spawned(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itThunderPokemonAttributes;
    (*ip).facing_dir = 0.0;
    (*ip).xDAC_itcmd_var0 = 0;
    it_802CCE28(gobj);
    ext::it_80279CDC(gobj, (*attrs).x0);
}

pub unsafe extern "C" fn it_802CCB10(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn it_802CCB14(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe extern "C" fn itThunder_UnkMotion1_Anim(gobj: *mut HSD_GObj) -> bool {
    let item = GET_ITEM(gobj);
    let mut jobj = GET_JOBJ(gobj);

    if jobj.is_null() {
        jobj = null_mut();
    } else {
        jobj = (*jobj).child;
    }
    if jobj.is_null() {
        jobj = null_mut();
    } else {
        jobj = (*jobj).child;
    }
    ext::it_8027A160(jobj, item);
    if ext::it_80272C6C(gobj) == 0 {
        it_802CCC68(gobj);
    }
    false
}

pub unsafe extern "C" fn itThunder_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    ext::it_8027A344(gobj);
}

pub unsafe extern "C" fn itThunder_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E15C(gobj, Some(it_802CCB10));
    false
}

pub unsafe extern "C" fn it_802CCBF8(gobj: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    let mut out: f32 = 1.0;
    if (*item).xDD4_itemVar.thunder.x64 != 0 {
        if (*item).xDD4_itemVar.thunder.x60 <= 0 {
            (*item).xDD4_itemVar.thunder.x64 = 0;
            ext::efSync_Spawn(0x460, gobj, (*(*item).xBBC_dynamicBoneTable).bones[2], &mut out);
        } else {
            (*item).xDD4_itemVar.thunder.x60 = (*item).xDD4_itemVar.thunder.x60 - 1;
        }
    }
}

pub unsafe extern "C" fn it_802CCC68(gobj: *mut HSD_GObj) {
    let item = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
    Item_SetEffectHitlagCallbacks(item);
    (*item).xDD4_itemVar.thunder.x68 = 0.0;
    (*item).xDB0_itcmd_var1 = 0;
}

pub unsafe extern "C" fn itThunder_UnkMotion2_Anim_inline2(gobj: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
    (*item).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*item).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
    (*item).xDD4_itemVar.thunder.x68 = 0.0;
    (*item).xDB0_itcmd_var1 = 0;
}

pub unsafe extern "C" fn itThunder_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    let item = GET_ITEM(gobj);
    ext::it_8027A160(itGetJObjGrandchild(gobj), item);
    if ext::it_80272C6C(gobj) == 0 {
        itThunder_UnkMotion2_Anim_inline2(gobj);
    }
    if (*item).pos.y > ext::Stage_GetBlastZoneTopOffset() {
        return true;
    }
    false
}

pub unsafe extern "C" fn itThunder_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    ext::it_8027A344(gobj);
    if (*item).xDAC_itcmd_var0 != 0 {
        let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itThunderPokemonAttributes;
        (*item).xDD4_itemVar.thunder.x68 = (*attrs).x4;
        (*item).xDAC_itcmd_var0 = 0;
        (*item).xDB0_itcmd_var1 = 1;
    }
    if (*item).xDB0_itcmd_var1 != 0 {
        let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itThunderPokemonAttributes;
        (*item).xDD4_itemVar.thunder.x68 += (*attrs).x8;
    }
    (*item).x40_vel.y = (*item).xDD4_itemVar.thunder.x68;
}

pub unsafe extern "C" fn itThunder_UnkMotion2_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn it_802CCE28(gobj: *mut Item_GObj) {
    Item_EnterAirStateWithHitlagAndStateDesc(gobj);
}

pub unsafe extern "C" fn itThunder_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    ext::it_80279FF8(gobj);
    false
}

pub unsafe extern "C" fn itThunder_UnkMotion0_Phys_inline2(gobj: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itThunderPokemonAttributes;
    (*item).xDD4_itemVar.thunder.x60 = (*attrs).xC as i32;
    (*item).xDD4_itemVar.thunder.x64 = 1;
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    (*item).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*item).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
    (*item).on_accessory = Some(it_802CCBF8);
}

pub unsafe extern "C" fn itThunder_UnkMotion0_Phys_inline(gobj: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    ext::it_80273454(gobj);
    itThunder_UnkMotion0_Phys_inline2(gobj);
    (*item).xDD1_flag.anon_0.set_b1(1);
}

pub unsafe extern "C" fn itThunder_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    if ext::it_8027A09C(gobj) != 0 {
        itThunder_UnkMotion0_Phys_inline(gobj);
    }
}

pub unsafe extern "C" fn itThunder_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8027A118(gobj, Some(it_802CCB10)) != 0
}
