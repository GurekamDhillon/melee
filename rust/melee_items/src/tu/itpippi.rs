//! Port of src/melee/it/kinds/itpippi.c: Clefairy (Pippi, Pokeball Pokemon, logic table 20), which
//! picks one of four random move states from the weights in `itPokemonAttributes`. Callees of
//! unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping
//! contraction) by tests/kinds_itpippi.rs.
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
    fn Item_80268D34(a0: *mut HSD_GObj, a1: *mut ItemStateDesc) => unsafe { crate::tu::item::Item_80268D34(a0, a1) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_80279FF8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279FF8(a0) };
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027A09C(a0) as _ };
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::it_279C::it_8027A118(a0, a1) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `ItemStateTable it_803F7FF8[]`
pub static mut it_803F7FF8: [ItemStateTable; 6] = [
    st_row(0, pred_b(itPippi_UnkMotion0_Anim), Some(itPippi_UnkMotion0_Phys), pred_b(itPippi_UnkMotion0_Coll)),
    st_row(1, pred_b(itPippi_UnkMotion1_Anim), Some(itPippi_UnkMotion1_Phys), pred_b(itPippi_UnkMotion1_Coll)),
    st_row(2, pred_b(itPippi_UnkMotion5_Anim), Some(itPippi_UnkMotion5_Phys), pred_b(itPippi_UnkMotion5_Coll)),
    st_row(3, pred_b(itPippi_UnkMotion5_Anim), Some(itPippi_UnkMotion5_Phys), pred_b(itPippi_UnkMotion5_Coll)),
    st_row(4, pred_b(itPippi_UnkMotion5_Anim), Some(itPippi_UnkMotion5_Phys), pred_b(itPippi_UnkMotion5_Coll)),
    st_row(5, pred_b(itPippi_UnkMotion5_Anim), Some(itPippi_UnkMotion5_Phys), pred_b(itPippi_UnkMotion5_Coll)),
];

pub unsafe extern "C" fn itPippi_Logic20_Spawned(gobj: *mut Item_GObj) {
    let it = (*gobj).user_data as *mut Item;
    let ap = (*it).xC4_article_data;
    let sa = (*ap).x4_specialAttributes as *mut itPokemonAttributes;

    (*it).facing_dir = 0.0;
    (*it).xDB4_itcmd_var2 = 0;
    (*it).xDB0_itcmd_var1 = 0;
    (*it).xDAC_itcmd_var0 = 0;
    ext::it_80279CDC(gobj, (*sa).x0);
    it_802D3590(gobj);
    ext::Item_8026AE84(it, 0x272A, 0x7F, 0x40);
}

/// 2D32D8
pub unsafe extern "C" fn it_802D32D8(_gobj: *mut Item_GObj) {}

/// 2D32DC
pub unsafe extern "C" fn it_802D32DC(gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe extern "C" fn itPippi_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    let it = (*gobj).user_data as *mut Item;

    if ext::it_80272C6C(gobj) == 0 {
        (*it).xDD4_itemVar.pokemon.timer = (*it).xDD4_itemVar.pokemon.timer.wrapping_sub(1);
        let timer = (*it).xDD4_itemVar.pokemon.timer;
        if (*it).xDD4_itemVar.pokemon.timer <= 0 {
            it_802D33F8(gobj);
        } else {
            ext::Item_80268D34(gobj, (*it).xD0_itemStateDesc);
        }
    }
    false
}

pub unsafe extern "C" fn itPippi_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let it = (*gobj).user_data as *mut Item;
    if (*it).ground_or_air == GA_Air {
        ext::it_80272860(gobj, (*(*it).xCC_item_attr).x10_fall_speed, (*(*it).xCC_item_attr).x14_fall_speed_max);
    }
}

pub unsafe extern "C" fn itPippi_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    let it = (*gobj).user_data as *mut Item;
    if (*it).ground_or_air == GA_Air {
        ext::it_8026E15C(gobj, Some(it_802D32D8));
    } else {
        ext::it_8026D62C(gobj, Some(it_802D32D8));
    }
    false
}

/// 2D33F8
pub unsafe extern "C" fn it_802D33F8(gobj: *mut Item_GObj) {
    let it = (*gobj).user_data as *mut Item;
    let ap = (*it).xC4_article_data;
    let sa = (*ap).x4_specialAttributes as *mut itPokemonAttributes;
    let rand = ext::HSD_Randi((*sa).max);
    let mut val = (*sa).xC;
    let state: i32;

    if rand < val {
        state = 2;
    } else {
        val = val.wrapping_add((*sa).x10);
        if rand < val {
            state = 3;
        } else if rand < (*sa).x14.wrapping_add(val) {
            state = 4;
        } else {
            state = 5;
        }
    }
    ext::Item_80268E5C(gobj, state, ITEM_ANIM_UPDATE);
    (*it).entered_hitlag = None;
    (*it).exited_hitlag = None;
}

pub unsafe extern "C" fn itPippi_UnkMotion5_Anim(gobj: *mut Item_GObj) -> bool {
    let it = (*gobj).user_data as *mut Item;
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268D34(gobj, (*it).xD0_itemStateDesc);
    }
    if (*it).xDAC_itcmd_var0 != 0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn itPippi_UnkMotion5_Phys(gobj: *mut Item_GObj) {
    let it = GET_ITEM(gobj);
    if (*it).ground_or_air == GA_Air {
        ext::it_80272860(gobj, (*(*it).xCC_item_attr).x10_fall_speed, (*(*it).xCC_item_attr).x14_fall_speed_max);
    }
}

pub unsafe extern "C" fn itPippi_UnkMotion5_Coll(gobj: *mut Item_GObj) -> bool {
    let it = GET_ITEM(gobj);
    if (*it).ground_or_air == GA_Air {
        ext::it_8026E15C(gobj, Some(it_802D32D8));
    } else {
        ext::it_8026D62C(gobj, Some(it_802D32D8));
    }
    false
}

/// 2D3590
pub unsafe extern "C" fn it_802D3590(gobj: *mut Item_GObj) {
    let it = (*gobj).user_data as *mut Item;
    let ap = (*it).xC4_article_data;
    let sa = (*ap).x4_specialAttributes as *mut itPokemonAttributes;

    ext::it_802762BC(it);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    (*it).entered_hitlag = None;
    (*it).exited_hitlag = None;
    (*it).xDD4_itemVar.pokemon.timer = (*sa).timer;
}

pub unsafe extern "C" fn itPippi_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    ext::it_80279FF8(gobj);
    false
}

pub unsafe extern "C" fn itPippi_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    if ext::it_8027A09C(gobj) != 0 {
        let it = (*gobj).user_data as *mut Item;
        ext::it_80273454(gobj);
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
        (*it).entered_hitlag = None;
        (*it).exited_hitlag = None;
    }
}

pub unsafe extern "C" fn itPippi_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8027A118(gobj, Some(it_802D32D8)) != 0
}
