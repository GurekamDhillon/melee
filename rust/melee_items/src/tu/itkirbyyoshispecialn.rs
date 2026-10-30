//! Native port of src/melee/it/kinds/itkirbyyoshispecialn.c: the Kirby copy-Yoshi-egg-swallow item
//! (`it_802F2D98` enters the held state attached to the owner, `it_802F2E7C` releases it, one empty
//! state `it_803F9490`). Callees in other TUs go through `ext` (hooks; unset panics "TODO").
//! Checked against the C (the whole TU, shipping contraction) by tests/tu_itkirbyyoshispecialn.rs.
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
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn ftKb_SpecialNYs_80109238(a0: *mut HSD_GObj) -> u32;
    fn ftLib_80086630(a0: *mut HSD_GObj, a1: u32) -> *mut HSD_JObj => unsafe { crate::tu::ftlib::ftLib_80086630(a0, a1) };
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

#[inline]
unsafe fn get_item(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

pub static mut it_803F9490: [ItemStateTable; 1] = [ItemStateTable {
    anim_id: -1,
    animated: Some(itKirbyyoshispecialn_UnkMotion0_Anim),
    physics_updated: Some(itKirbyyoshispecialn_UnkMotion0_Phys),
    collided: Some(itKirbyyoshispecialn_UnkMotion0_Coll),
}];

pub unsafe extern "C" fn it_802F2D98(gobj: *mut Item_GObj, owner_gobj: *mut Fighter_GObj) {
    melee_core::ct!("it_802F2D98");
    let item = get_item(gobj);
    (*item).atk_victim = owner_gobj;
    (*item).grab_victim = owner_gobj;
    ext::it_8026C220(gobj, owner_gobj);
    (*item).xDD0_flag.anon_0.set_b1(0);
    (*item).xBC_itemStateContainer = it_803F9490.as_mut_ptr();
    ext::it_80274F48(gobj, (*(*(*item).xC4_article_data).x10_modelDesc).x8_bone_attach_id, owner_gobj, ext::ftKb_SpecialNYs_80109238(owner_gobj));
    ext::Item_80268E5C(gobj, 0, 0);
    ext::it_802762BC(item);
    ext::it_802756D0(gobj);
    ext::it_80274ECC(gobj, 1);
    ext::it_80274C88(gobj);
    ext::it_8026BD54(gobj);
    ext::it_80273408(gobj);
}

pub unsafe extern "C" fn itKirbyyoshispecialn_UnkMotion0_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKirbyyoshispecialn_UnkMotion0_Anim");
    0
}

pub unsafe extern "C" fn itKirbyyoshispecialn_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itKirbyyoshispecialn_UnkMotion0_Phys");}

pub unsafe extern "C" fn itKirbyyoshispecialn_UnkMotion0_Coll(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKirbyyoshispecialn_UnkMotion0_Coll");
    0
}

pub unsafe extern "C" fn it_802F2E7C(gobj: *mut Item_GObj, has_destroy_type: i32) {
    melee_core::ct!("it_802F2E7C");
    let item = get_item(gobj);
    ext::it_80275070(gobj, (*(*(*item).xC4_article_data).x10_modelDesc).x8_bone_attach_id);
    let jobj = ext::ftLib_80086630((*item).grab_victim, ext::ftKb_SpecialNYs_80109238((*item).grab_victim));
    ext::lb_8000B1CC(jobj, null_mut(), &mut (*item).pos);
    ext::it_8027B4A4((*item).grab_victim, gobj);
    ext::it_8027B378((*item).grab_victim, gobj, 0.0);
    if has_destroy_type != 0 {
        (*item).destroy_type = 0;
    }
    (*item).atk_victim = null_mut();
    (*item).grab_victim = null_mut();
    ext::it_8027CE44(gobj);
    ext::Item_8026A8EC(gobj);
}
