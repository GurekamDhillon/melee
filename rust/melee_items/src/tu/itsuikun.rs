//! Port of src/melee/it/kinds/itsuikun.c: the Squirtle Pokemon (logic table 14). `xDAC_itcmd_var0` is
//! the "spawn effect" request, `xDB0_itcmd_var1` the "stop effects" request and `xDB4_itcmd_var2` the
//! "effect running" flag, all set by the item script. Callees of unported TUs go through `ext` (unset:
//! panics "TODO"). Checked against the C (whole TU, shipping contraction) by tests/kinds_itsuikun.rs.
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
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec);
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efLib_DestroyAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_DestroyAll(a0) };
    fn efLib_PauseAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_PauseAll(a0) };
    fn efLib_ResumeAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_ResumeAll(a0) };
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj, a3: *mut f32) -> *mut c_void => unsafe { melee_world::tu_ext::efSync_Spawn(a0, a1, a2, a3) };
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

/// `Item_SetEffectHitlagCallbacks` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_SetEffectHitlagCallbacks(ip: *mut Item) {
    (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}

/// `Item_EnterAirStateWithHitlag` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_EnterAirStateWithHitlag(gobj: *mut Item_GObj, msid: i32) {
    let item = GET_ITEM(gobj);
    ext::it_802762BC(item);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE);
    (*item).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*item).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}

/// `ItemStateTable it_803F7E08[]`
pub static mut it_803F7E08: [ItemStateTable; 2] = [
    st_row(0, pred_b(itSuikun_UnkMotion0_Anim), Some(itSuikun_UnkMotion0_Phys), pred_b(itSuikun_UnkMotion0_Coll)),
    st_row(-1, pred_b(itSuikun_UnkMotion1_Anim), Some(itSuikun_UnkMotion1_Phys), pred_b(itSuikun_UnkMotion1_Coll)),
];

pub unsafe extern "C" fn itSuikun_Logic14_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itSuikun_Logic14_Spawned");
    let it = (*gobj).user_data as *mut Item;
    let sa = (*(*it).xC4_article_data).x4_specialAttributes as *mut itPokemonAttributes;

    ext::it_80279C48(gobj);
    (*it).xDAC_itcmd_var0 = 0;
    (*it).xDB0_itcmd_var1 = 0;
    (*it).xDB4_itcmd_var2 = 0;
    ext::it_80279CDC(gobj, (*sa).x0);
    it_802CFFAC(gobj);
}

/// 2CFCB4
pub unsafe extern "C" fn it_802CFCB4(_gobj: *mut Item_GObj) { melee_core::ct!("it_802CFCB4");}

/// 2CFCB8
pub unsafe extern "C" fn it_802CFCB8(gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802CFCB8");
    ext::it_8026B894(gobj, ref_gobj);
}

/// 2CFCD8
pub unsafe extern "C" fn it_802CFCD8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CFCD8");
    let it = GET_ITEM(gobj);

    ext::Camera_RequestQuake(QuakeKind_Large as u32, null_mut());
    ext::Item_8026AE84(it, 0x9, 0x7F, 0x40);
    ext::it_80273454(gobj);
    it_802CFD3C(gobj);
}

/// 2CFD3C
pub unsafe extern "C" fn it_802CFD3C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CFD3C");
    let it = (*gobj).user_data as *mut Item;
    let ap = (*it).xC4_article_data;
    let sa = (*ap).x4_specialAttributes as *mut itPokemonAttributes;

    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    Item_SetEffectHitlagCallbacks(it);
    (*it).on_accessory = Some(it_802CFF30);
    (*it).xDD4_itemVar.pokemon.timer = (*sa).timer;
}

pub unsafe extern "C" fn itSuikun_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSuikun_UnkMotion0_Anim");
    let it = (*gobj).user_data as *mut Item;
    let mut sfx: i32 = 0;

    if (*it).xDB0_itcmd_var1 != 0 {
        (*it).xDB0_itcmd_var1 = 0;
        (*it).xDB4_itcmd_var2 = 0;
        ext::efLib_DestroyAll(gobj);
    }
    if ext::it_80272C6C(gobj) == 0 {
        return true;
    }
    if (*it).xDB4_itcmd_var2 != 0 {
        (*it).xDD4_itemVar.pokemon.timer = (*it).xDD4_itemVar.pokemon.timer.wrapping_sub(1);
        let timer = (*it).xDD4_itemVar.pokemon.timer;
        if timer == 0 {
            let sa = (*(*it).xC4_article_data).x4_specialAttributes as *mut itPokemonAttributes;
            match ext::HSD_Randi(3) {
                0 => sfx = 0x274A,
                1 => sfx = 0x274B,
                2 => sfx = 0x274C,
                _ => {}
            }
            ext::Item_8026AE84(it, sfx, 0x7F, 0x40);
            (*it).xDD4_itemVar.pokemon.timer = (*sa).timer;
        }
    }
    false
}

pub unsafe extern "C" fn itSuikun_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSuikun_UnkMotion0_Phys");
    let it = GET_ITEM(gobj);
    if (*it).ground_or_air == GA_Air {
        ext::it_80272860(gobj, (*(*it).xCC_item_attr).x10_fall_speed, (*(*it).xCC_item_attr).x14_fall_speed_max);
    }
}

pub unsafe extern "C" fn itSuikun_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSuikun_UnkMotion0_Coll");
    let it = GET_ITEM(gobj);
    if (*it).ground_or_air == GA_Air {
        ext::it_8026E248(gobj, Some(it_802CFCB4));
    } else {
        ext::it_8026D62C(gobj, Some(it_802CFCB4));
    }
    false
}

/// 2CFF30
pub unsafe extern "C" fn it_802CFF30(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CFF30");
    let it = (*gobj).user_data as *mut Item;

    if (*it).xDAC_itcmd_var0 != 0 {
        let jp = (*gobj).hsd_obj as *mut HSD_JObj;
        let mut v: f32 = 1.0;

        (*it).xDAC_itcmd_var0 = 0;
        (*it).xDB4_itcmd_var2 = 1;
        ext::efSync_Spawn(0x469, gobj, jp, &mut v);
        ext::Item_8026AE84(it, 0x2749, 0x7f, 0x40);
    }
}

/// 2CFFAC
pub unsafe extern "C" fn it_802CFFAC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CFFAC");
    Item_EnterAirStateWithHitlag(gobj, 1);
}

pub unsafe extern "C" fn itSuikun_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSuikun_UnkMotion1_Anim");
    ext::it_80279FF8(gobj);

    false
}

pub unsafe extern "C" fn itSuikun_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSuikun_UnkMotion1_Phys");
    ext::it_8027A09C(gobj);
}

pub unsafe extern "C" fn itSuikun_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSuikun_UnkMotion1_Coll");
    ext::it_8027A118(gobj, Some(it_802CFCD8)) != 0
}
