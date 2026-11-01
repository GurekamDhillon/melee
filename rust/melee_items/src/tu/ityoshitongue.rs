//! Port of src/melee/it/kinds/ityoshitongue.c: Yoshi's tongue-grab item (logic table 103 helper). Callees of
//! unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction)
//! by tests/kinds_ityoshitongue.rs.
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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn ftLib_80086630(a0: *mut HSD_GObj, a1: u32) -> *mut HSD_JObj => unsafe { crate::tu::ftlib::ftLib_80086630(a0, a1) };
    fn ftYs_SpecialN_GetBoneIndex(a0: *mut HSD_GObj) -> u32;
    fn it_8026BD54(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd54(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026C220(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026C220(a0, a1) };
    fn it_80273408(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273408(a0) };
    fn it_80274C88(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80274c88(&mut *((*a0).user_data as *mut Item)) };
    fn it_80274ECC(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::itlib::it_80274ecc(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_80274F48(a0: *mut HSD_GObj, a1: i32, a2: *mut HSD_GObj, a3: u32) => unsafe { crate::tu::it_2725::it_80274F48(a0, a1, a2, a3) };
    fn it_80275070(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::it_2725::it_80275070(a0, a1) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_8027B378(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: f32) => unsafe { crate::tu::it_279C::it_8027B378(a0, a1, a2) };
    fn it_8027B4A4(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_8027B4A4(a0, a1) };
    fn it_8027CE44(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CE44(a0) };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `ItemStateTable it_803F9480[]`
pub static mut it_803F9480: [ItemStateTable; 1] = [st_row(
    -1,
    pred_b!(it_2F2B_UnkMotion0_Anim),
    Some(it_2F2B_UnkMotion0_Phys),
    pred_b!(it_2F2B_UnkMotion0_Coll),
)];

pub unsafe extern "C" fn it_802F2BFC(gobj: *mut Item_GObj, owner_gobj: *mut Fighter_GObj) {
    melee_core::ct!("it_802F2BFC");
    let item = GET_ITEM(gobj);
    (*item).atk_victim = owner_gobj;
    (*item).grab_victim = owner_gobj;
    ext::it_8026C220(gobj, owner_gobj);
    (*item).xDD0_flag.anon_0.set_b1(0);
    (*item).xBC_itemStateContainer = it_803F9480.as_mut_ptr();
    ext::it_80274F48(
        gobj,
        (*(*(*item).xC4_article_data).x10_modelDesc).x8_bone_attach_id,
        owner_gobj,
        ext::ftYs_SpecialN_GetBoneIndex(owner_gobj),
    );
    ext::Item_80268E5C(gobj, 0, 0);
    ext::it_802762BC(item);
    ext::it_802756D0(gobj);
    ext::it_80274ECC(gobj, 1);
    ext::it_80274C88(gobj);
    ext::it_8026BD54(gobj);
    ext::it_80273408(gobj);
}

pub unsafe extern "C" fn it_2F2B_UnkMotion0_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2F2B_UnkMotion0_Anim");
    false
}

pub unsafe extern "C" fn it_2F2B_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("it_2F2B_UnkMotion0_Phys");}

pub unsafe extern "C" fn it_2F2B_UnkMotion0_Coll(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2F2B_UnkMotion0_Coll");
    false
}

pub unsafe extern "C" fn it_802F2CE0(gobj: *mut Item_GObj, clear_destroy_type: bool) {
    melee_core::ct!("it_802F2CE0");
    let item = GET_ITEM(gobj);
    ext::it_80275070(gobj, (*(*(*item).xC4_article_data).x10_modelDesc).x8_bone_attach_id);
    ext::lb_8000B1CC(
        ext::ftLib_80086630((*item).grab_victim, ext::ftYs_SpecialN_GetBoneIndex((*item).grab_victim)),
        null_mut(),
        &mut (*item).pos,
    );
    ext::it_8027B4A4((*item).grab_victim, gobj);
    ext::it_8027B378((*item).grab_victim, gobj, 0.0);
    if clear_destroy_type {
        (*item).destroy_type = 0;
    }
    (*item).atk_victim = null_mut();
    (*item).grab_victim = null_mut();
    ext::it_8027CE44(gobj);
    ext::Item_8026A8EC(gobj);
}
