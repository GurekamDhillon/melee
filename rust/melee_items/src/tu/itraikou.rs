//! Port of src/melee/it/kinds/itraikou.c: Raikou (Pokemon item). Callees of unported TUs go through `ext`
//! (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by tests/kinds_itraikou.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::itfwd::it_maril_unk_motion2_anim;
#[allow(unused_imports)]
use crate::itfwd::it_zgshell_logic11_reflected;
#[allow(unused_imports)]
use crate::layout::HSD_GObj;
#[allow(unused_imports)]
use crate::layout::Item;
#[allow(unused_imports)]
use crate::layout::ItemCommonData;
#[allow(unused_imports)]
use crate::helpers::decrease_lifetimer;
#[allow(unused_imports)]
use crate::itkinds::it_802d5884;
#[allow(unused_imports)]
use crate::itkinds::it_802e657c;
#[allow(unused_imports)]
use crate::itkinds::it_chicorita_phys;
#[allow(unused_imports)]
use crate::itkinds::it_harisen_logic24_dmg_dealt;
#[allow(unused_imports)]
use crate::itkinds::it_lipstick_logic23_dmg_dealt;
#[allow(unused_imports)]
use crate::itkinds::it_lizardon_unk_motion1_phys;
#[allow(unused_imports)]
use crate::itkinds::it_maril_unk_motion2_anim_inline;
#[allow(unused_imports)]
use crate::itkinds::it_maril_unk_motion2_phys;
#[allow(unused_imports)]
use crate::itlib::it_8026b3a8;
#[allow(unused_imports)]
use crate::itlib::it_8026b724;
#[allow(unused_imports)]
use crate::itlib::it_8026b73c;
#[allow(unused_imports)]
use crate::itlib::it_8026b894;
#[allow(unused_imports)]
use crate::itlib::it_80273030;
#[allow(unused_imports)]
use crate::itlib::it_80273130;
#[allow(unused_imports)]
use crate::itlib::it_802751d8;
#[allow(unused_imports)]
use crate::itlib::it_80275870;
#[allow(unused_imports)]
use crate::itlib3::it_8027a344;
#[allow(unused_imports)]
use crate::itlib3::it_8027b730;
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

/// `ItemStateTable it_803F7DE8[]`
pub static mut it_803F7DE8: [ItemStateTable; 2] = [
    st_row(0, pred_b!(itRaikou_UnkMotion0_Anim), Some(itRaikou_UnkMotion0_Phys), pred_b!(itRaikou_UnkMotion0_Coll)),
    st_row(-1, pred_b!(itRaikou_UnkMotion1_Anim), Some(itRaikou_UnkMotion1_Phys), pred_b!(itRaikou_UnkMotion1_Coll)),
];

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

pub unsafe extern "C" fn it_802CF814(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CF814");
    let it = (*gobj).user_data as *mut Item;
    let sa = (*(*it).xC4_article_data).x4_specialAttributes as *mut itPokemonAttributes;

    ext::it_80279C48(gobj);
    (*it).xDAC_itcmd_var0 = 0;
    (*it).xDB0_itcmd_var1 = 0;
    (*it).xDB4_itcmd_var2 = 0;
    ext::it_80279CDC(gobj, (*sa).x0);
    it_802CFB78(gobj);
}

pub unsafe extern "C" fn it_802CF880(_gobj: *mut Item_GObj) { melee_core::ct!("it_802CF880");}

pub unsafe extern "C" fn it_802CF884(gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802CF884");
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe extern "C" fn it_802CF8A4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CF8A4");
    let it = GET_ITEM(gobj);

    ext::Camera_RequestQuake(QuakeKind_Large, null_mut());
    ext::Item_8026AE84(it, 0x9, 0x7F, 0x40);
    ext::it_80273454(gobj);
    it_802CF908(gobj);
}

pub unsafe extern "C" fn it_802CF908(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CF908");
    let it = (*gobj).user_data as *mut Item;
    let ap = (*it).xC4_article_data;
    let sa = (*ap).x4_specialAttributes as *mut itPokemonAttributes;

    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    Item_SetEffectHitlagCallbacks(it);
    (*it).on_accessory = Some(it_802CFAFC);
    (*it).xDD4_itemVar.pokemon.timer = (*sa).timer;
}

pub unsafe extern "C" fn itRaikou_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itRaikou_UnkMotion0_Anim");
    let it = (*gobj).user_data as *mut Item;
    let sa: *mut itPokemonAttributes;
    let timer: i32;
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
        timer = (*it).xDD4_itemVar.pokemon.timer;
        if timer == 0 {
            sa = (*(*it).xC4_article_data).x4_specialAttributes as *mut itPokemonAttributes;
            match ext::HSD_Randi(3) {
                0 => sfx = 0x2746,
                1 => sfx = 0x2747,
                2 => sfx = 0x2748,
                _ => {}
            }
            ext::Item_8026AE84(it, sfx, 0x7F, 0x40);
            (*it).xDD4_itemVar.pokemon.timer = (*sa).timer;
        }
    }
    false
}

pub unsafe extern "C" fn itRaikou_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itRaikou_UnkMotion0_Phys");
    let it = GET_ITEM(gobj);
    if (*it).ground_or_air == GA_Air {
        ext::it_80272860(gobj, (*(*it).xCC_item_attr).x10_fall_speed, (*(*it).xCC_item_attr).x14_fall_speed_max);
    }
}

pub unsafe extern "C" fn itRaikou_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itRaikou_UnkMotion0_Coll");
    let it = GET_ITEM(gobj);
    if (*it).ground_or_air == GA_Air {
        ext::it_8026E248(gobj, Some(it_802CF880));
    } else {
        ext::it_8026D62C(gobj, Some(it_802CF880));
    }
    false
}

pub unsafe extern "C" fn it_802CFAFC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CFAFC");
    let it = (*gobj).user_data as *mut Item;

    if (*it).xDAC_itcmd_var0 != 0 {
        let jp = (*gobj).hsd_obj as *mut HSD_JObj;
        let mut v: f32 = 1.0;

        (*it).xDAC_itcmd_var0 = 0;
        (*it).xDB4_itcmd_var2 = 1;
        ext::efSync_Spawn(0x46D, gobj, jp, &mut v);
        ext::Item_8026AE84(it, 0x2745, 0x7f, 0x40);
    }
}

pub unsafe extern "C" fn it_802CFB78(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CFB78");
    Item_EnterAirStateWithHitlag(gobj, 1);
}

pub unsafe extern "C" fn itRaikou_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itRaikou_UnkMotion1_Anim");
    ext::it_80279FF8(gobj);
    false
}

pub unsafe extern "C" fn itRaikou_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itRaikou_UnkMotion1_Phys");
    ext::it_8027A09C(gobj);
}

pub unsafe extern "C" fn itRaikou_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itRaikou_UnkMotion1_Coll");
    ext::it_8027A118(gobj, Some(it_802CF8A4)) != 0
}

// ---- folded from itfwd.rs (consolidation steps 21/22) ----
/// `it_802CF884` (src/melee/it/kinds/itraikou.c): `it_8026B894(..)`.
pub fn it_802cf884(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802cf884");
    it_8026b894(ip, ref_gobj);
}

// ---- folded from itfwd6.rs (consolidation steps 21/22) ----
/// `it_802CF880` (src/melee/it/kinds/itraikou.c): `return;`.
pub fn it_802cf880(_ip: &mut Item) { melee_core::ct!("it_802cf880");}
