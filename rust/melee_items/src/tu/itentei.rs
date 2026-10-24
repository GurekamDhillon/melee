//! Native port of src/melee/it/kinds/itentei.c: the Entei pokemon item (spawn, its roar timer,
//! the state callbacks and the state table `it_803F7DC8`). Callees in other TUs go through `ext`
//! (hooks; unset panics "TODO"). Checked against the C (the whole TU, shipping contraction) by
//! tests/tu_itentei.rs.
//!
//! Header inlines compiled into the TU: `Item_SetEffectHitlagCallbacks`,
//! `Item_EnterAirStateWithHitlag`. The variadic `efSync_Spawn` is
//! `ext::efSync_Spawn_N(gfx, gobj, &[args])` (the oracle counts the arguments).
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
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec);
    fn HSD_Randi(a0: i32) -> i32;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efLib_DestroyAll(a0: *mut HSD_GObj);
    fn efLib_PauseAll(a0: *mut HSD_GObj);
    fn efLib_ResumeAll(a0: *mut HSD_GObj);
    fn efSync_Spawn_N(a0: i32, a1: *mut HSD_GObj, a2: &[*mut c_void]) -> *mut c_void;
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026E248(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E248(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80279C48(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279C48(a0) };
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

/// `Item_EnterAirStateWithHitlag`.
#[inline]
unsafe fn Item_EnterAirStateWithHitlag(gobj: *mut Item_GObj, msid: i32) {
    let item = get_item(gobj);
    ext::it_802762BC(item);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE as _);
    Item_SetEffectHitlagCallbacks(item);
}

pub static mut it_803F7DC8: [ItemStateTable; 2] = [
    ItemStateTable { anim_id: 0, animated: Some(itEntei_UnkMotion0_Anim), physics_updated: Some(itEntei_UnkMotion0_Phys), collided: Some(itEntei_UnkMotion0_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itEntei_UnkMotion1_Anim), physics_updated: Some(itEntei_UnkMotion1_Phys), collided: Some(itEntei_UnkMotion1_Coll) },
];

pub unsafe extern "C" fn it_802CF3E0(gobj: *mut Item_GObj) {
    let it = (*gobj).user_data as *mut Item;
    let ap = (*it).xC4_article_data;
    let sa = (*ap).x4_specialAttributes as *mut itPokemonAttributes;
    ext::it_80279C48(gobj);
    (*it).xDAC_itcmd_var0 = 0;
    (*it).xDB0_itcmd_var1 = 0;
    (*it).xDB4_itcmd_var2 = 0;
    ext::it_80279CDC(gobj, (*sa).x0);
    it_802CF744(gobj);
}

pub unsafe extern "C" fn it_802CF44C(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn it_802CF450(gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe extern "C" fn it_802CF470(gobj: *mut Item_GObj) {
    let it = get_item(gobj);
    ext::Camera_RequestQuake(QuakeKind_Large as _, null_mut());
    ext::Item_8026AE84(it, 0x9, 0x7F, 0x40);
    ext::it_80273454(gobj);
    it_802CF4D4(gobj);
}

pub unsafe extern "C" fn it_802CF4D4(gobj: *mut Item_GObj) {
    let it = (*gobj).user_data as *mut Item;
    let ap = (*it).xC4_article_data;
    let sa = (*ap).x4_specialAttributes as *mut itPokemonAttributes;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as _);
    Item_SetEffectHitlagCallbacks(it);
    (*it).on_accessory = Some(it_802CF6C8);
    (*it).xDD4_itemVar.pokemon.timer = (*sa).timer;
}

pub unsafe extern "C" fn itEntei_UnkMotion0_Anim(gobj: *mut Item_GObj) -> i32 {
    let it = (*gobj).user_data as *mut Item;
    if (*it).xDB0_itcmd_var1 != 0 {
        (*it).xDB0_itcmd_var1 = 0;
        (*it).xDB4_itcmd_var2 = 0;
        ext::efLib_DestroyAll(gobj);
    }
    if ext::it_80272C6C(gobj) == 0 {
        return 1;
    }
    if (*it).xDB4_itcmd_var2 != 0 {
        (*it).xDD4_itemVar.pokemon.timer -= 1;
        let timer = (*it).xDD4_itemVar.pokemon.timer;
        if timer == 0 {
            let sa = (*(*it).xC4_article_data).x4_specialAttributes as *mut itPokemonAttributes;
            let sfx = match ext::HSD_Randi(3) {
                0 => 0x2742,
                1 => 0x2743,
                2 => 0x2744,
                // the C leaves `sfx` unset (a zero-initialised local in the shipping build)
                _ => 0,
            };
            ext::Item_8026AE84(it, sfx, 0x7F, 0x40);
            (*it).xDD4_itemVar.pokemon.timer = (*sa).timer;
        }
    }
    0
}

pub unsafe extern "C" fn itEntei_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let it = get_item(gobj);
    if (*it).ground_or_air == GA_Air {
        ext::it_80272860(gobj, (*(*it).xCC_item_attr).x10_fall_speed, (*(*it).xCC_item_attr).x14_fall_speed_max);
    }
}

pub unsafe extern "C" fn itEntei_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    let it = get_item(gobj);
    if (*it).ground_or_air == GA_Air {
        ext::it_8026E248(gobj, Some(it_802CF44C));
    } else {
        ext::it_8026D62C(gobj, Some(it_802CF44C));
    }
    0
}

pub unsafe extern "C" fn it_802CF6C8(gobj: *mut Item_GObj) {
    let it = (*gobj).user_data as *mut Item;
    if (*it).xDAC_itcmd_var0 != 0 {
        let jp = (*gobj).hsd_obj as *mut HSD_JObj;
        let mut v: f32 = 1.0;
        (*it).xDAC_itcmd_var0 = 0;
        (*it).xDB4_itcmd_var2 = 1;
        ext::efSync_Spawn_N(0x468, gobj, &[jp as *mut c_void, &mut v as *mut f32 as *mut c_void]);
        ext::Item_8026AE84(it, 0x2741, 0x7f, 0x40);
    }
}

pub unsafe extern "C" fn it_802CF744(gobj: *mut Item_GObj) {
    Item_EnterAirStateWithHitlag(gobj, 1);
}

pub unsafe extern "C" fn itEntei_UnkMotion1_Anim(gobj: *mut Item_GObj) -> i32 {
    ext::it_80279FF8(gobj);
    0
}

pub unsafe extern "C" fn itEntei_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    ext::it_8027A09C(gobj);
}

pub unsafe extern "C" fn itEntei_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8027A118(gobj, Some(it_802CF470))
}
