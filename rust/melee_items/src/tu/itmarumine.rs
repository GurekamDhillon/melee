//! Port of src/melee/it/kinds/itmarumine.c: Marumine / Voltorb (Pokeball Pokemon, logic table 16;
//! `it_803F7E78`). `pokemon.timer` is the fuse, `pokemon.x64` the frames until the next spark effect.
//! Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU,
//! shipping contraction) by tests/kinds_itmarumine.rs.
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
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efLib_PauseAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_PauseAll(a0) };
    fn efLib_ResumeAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_ResumeAll(a0) };
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj, a3: *mut f32) -> *mut c_void => unsafe { melee_world::tu_ext::efSync_Spawn(a0, a1, a2, a3) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BD24(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd24(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E71C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E71C(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80274250(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_2725::it_80274250(a0, a1 as _) };
    fn it_8027518C(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_8027518C(a0) };
    fn it_802751D8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_802751d8(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275444(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275444(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_80279FF8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279FF8(a0) };
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027A09C(a0) as _ };
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::it_279C::it_8027A118(a0, a1) as _ };
    fn lb_8000BA0C(a0: *mut HSD_JObj, a1: f32) => unsafe { melee_runtime::tu::lb_00B0::lb_8000BA0C(a0, a1) };
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

/// `static Vec3 const it_803B86C8`
static it_803B86C8: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

/// `ItemStateTable it_803F7E78[]`
pub static mut it_803F7E78: [ItemStateTable; 7] = [
    st_row(0, pred_b(itMarumine_UnkMotion0_Anim), Some(itMarumine_UnkMotion0_Phys), pred_b(itMarumine_UnkMotion0_Coll)),
    st_row(1, pred_b(itMarumine_UnkMotion1_Anim), Some(itMarumine_UnkMotion1_Phys), pred_b(itMarumine_UnkMotion1_Coll)),
    st_row(2, pred_b(itMarumine_UnkMotion2_Anim), Some(itMarumine_UnkMotion2_Phys), pred_b(itMarumine_UnkMotion2_Coll)),
    st_row(3, pred_b(itMarumine_UnkMotion3_Anim), Some(itMarumine_UnkMotion3_Phys), pred_b(itMarumine_UnkMotion3_Coll)),
    st_row(4, pred_b(itMarumine_UnkMotion4_Anim), Some(itMarumine_UnkMotion4_Phys), pred_b(itMarumine_UnkMotion4_Coll)),
    st_row(5, pred_b(itMarumine_UnkMotion5_Anim), Some(itMarumine_UnkMotion5_Phys), pred_b(itMarumine_UnkMotion5_Coll)),
    st_row(6, pred_b(itMarumine_UnkMotion6_Anim), Some(itMarumine_UnkMotion6_Phys), pred_b(itMarumine_UnkMotion6_Coll)),
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

/// `HSD_JObjSetFlagsAll` is out of line (an `ext` hook).
/// 2D09D0
pub unsafe extern "C" fn it_802D09D0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D09D0");
    let ip = GET_ITEM(gobj);
    if (*ip).xDC8_word.flags.x13() == 0 && (*ip).msid != 6 {
        it_802D1204(gobj);
    }
}

pub unsafe extern "C" fn itMarumine_Logic16_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itMarumine_Logic16_Spawned");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPokemonAttributes;
    (*ip).facing_dir = 0.0;
    (*ip).xDC8_word.flags.set_x0(1);
    (*ip).xDD4_itemVar.pokemon.timer = 0xB4i32.wrapping_sub((*attr).max);
    ext::it_80279CDC(gobj, (*attr).x0);
    ext::it_80273454(gobj);
    it_802D1320(gobj);
    (*ip).xDC8_word.flags.set_x0(1);
    (*ip).xDD4_itemVar.pokemon.x64 = (*attr).xC;
    (*ip).xDAC_itcmd_var0 = 1;
}

/// 2D0AAC
pub unsafe extern "C" fn it_802D0AAC(_gobj: *mut Item_GObj) { melee_core::ct!("it_802D0AAC");}

/// 2D0AB0
pub unsafe extern "C" fn it_802D0AB0(gobj: *mut Item_GObj, r: *mut Item_GObj) {
    melee_core::ct!("it_802D0AB0");
    ext::it_8026B894(gobj, r);
}

/// 2D0AD0
pub unsafe extern "C" fn it_802D0AD0(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802D0AD0");
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 4 {
        it_802D1204(gobj);
    }
    false
}

/// 2D0B04
pub unsafe extern "C" fn it_802D0B04(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D0B04");
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    (*ip).x5D0_animFrameSpeed = 1.0;
    ext::lb_8000BA0C(jobj, (*ip).x5D0_animFrameSpeed);
    ext::Item_80268E5C(gobj, 1, ITEM_UNK_0x1);
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn itMarumine_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarumine_UnkMotion1_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        it_802D100C(gobj);
    }
    false
}

pub unsafe extern "C" fn itMarumine_UnkMotion1_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itMarumine_UnkMotion1_Phys");}

pub unsafe extern "C" fn itMarumine_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarumine_UnkMotion1_Coll");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPokemonAttributes;
    if ext::it_8026DA08(gobj) == 0 {
        if (*ip).x5CC_currentAnimFrame >= (*attr).timer as f32 {
            it_802D0C44(gobj);
        }
    }
    false
}

/// 2D0C44
pub unsafe extern "C" fn it_802D0C44(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D0C44");
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    (*ip).x5D0_animFrameSpeed = 0.0;
    ext::lb_8000BA0C(jobj, (*ip).x5D0_animFrameSpeed);
    ext::Item_80268E5C(gobj, 2, ITEM_UNK_0x1);
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn itMarumine_UnkMotion2_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarumine_UnkMotion2_Anim");
    false
}

pub unsafe extern "C" fn itMarumine_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itMarumine_UnkMotion2_Phys");
    let ip = GET_ITEM(gobj);
    let attr = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itMarumine_UnkMotion2_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarumine_UnkMotion2_Coll");
    ext::it_8026E15C(gobj, Some(it_802D0B04));
    false
}

/// 2D0D18
pub unsafe extern "C" fn it_802D0D18(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D0D18");
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 3, ITEM_UNK_0x1);
    (*ip).on_accessory = Some(it_802D0DBC);
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn itMarumine_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarumine_UnkMotion3_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        it_802D1204(gobj);
    }
    false
}

pub unsafe extern "C" fn itMarumine_UnkMotion3_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itMarumine_UnkMotion3_Phys");}

pub unsafe extern "C" fn itMarumine_UnkMotion3_Coll(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarumine_UnkMotion3_Coll");
    false
}

/// 2D0DBC
pub unsafe extern "C" fn it_802D0DBC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D0DBC");
    let mut sp10: f32;
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.pokemon.x64 = (*ip).xDD4_itemVar.pokemon.x64.wrapping_sub(1);
    if (*ip).xDD4_itemVar.pokemon.x64 == 0 {
        let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPokemonAttributes;
        sp10 = 1.0;
        ext::efSync_Spawn(0x471, gobj, (*(*ip).xBBC_dynamicBoneTable).bones[3], &mut sp10);
        (*ip).xDD4_itemVar.pokemon.x64 = (*attr).xC;
    }
}

/// 2D0E30
pub unsafe extern "C" fn it_802D0E30(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D0E30");
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 4, ITEM_UNK_0x1);
    ext::it_8026B3A8(gobj);
    Item_SetEffectHitlagCallbacks(ip);
}

/// 2D0E90
pub unsafe extern "C" fn it_802D0E90(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D0E90");
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 4, ITEM_UNK_0x1);
    ext::it_8026B3A8(gobj);
    (*ip).on_accessory = Some(fn_802D0F98);
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn itMarumine_UnkMotion4_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarumine_UnkMotion4_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        it_802D1204(gobj);
    }
    false
}

pub unsafe extern "C" fn itMarumine_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itMarumine_UnkMotion4_Phys");
    let ip = GET_ITEM(gobj);
    let attr = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itMarumine_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarumine_UnkMotion4_Coll");
    ext::it_8026E71C(gobj, Some(it_802D1204));
    false
}

/// 2D0F98
pub unsafe extern "C" fn fn_802D0F98(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_802D0F98");
    let mut sp10: f32;
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.pokemon.x64 = (*ip).xDD4_itemVar.pokemon.x64.wrapping_sub(1);
    if (*ip).xDD4_itemVar.pokemon.x64 == 0 {
        let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPokemonAttributes;
        sp10 = 1.0;
        ext::efSync_Spawn(0x471, gobj, (*(*ip).xBBC_dynamicBoneTable).bones[3], &mut sp10);
        (*ip).xDD4_itemVar.pokemon.x64 = (*attr).xC;
    }
}

/// 2D100C
pub unsafe extern "C" fn it_802D100C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D100C");
    let ip = GET_ITEM(gobj);
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
    (*ip).on_accessory = Some(it_802D1140);
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn itMarumine_UnkMotion5_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarumine_UnkMotion5_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        it_802D1204(gobj);
    }
    false
}

pub unsafe extern "C" fn itMarumine_UnkMotion5_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itMarumine_UnkMotion5_Phys");
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Air {
        let attr = (*ip).xCC_item_attr;
        ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
    }
}

pub unsafe extern "C" fn itMarumine_UnkMotion5_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarumine_UnkMotion5_Coll");
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Air {
        ext::it_8026E15C(gobj, Some(it_802D0AAC));
    } else {
        ext::it_8026D62C(gobj, Some(it_802D0AAC));
    }
    false
}

/// 2D1140
pub unsafe extern "C" fn it_802D1140(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D1140");
    let mut sp10: f32;
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.pokemon.timer = (*ip).xDD4_itemVar.pokemon.timer.wrapping_sub(1);
    if (*ip).xDD4_itemVar.pokemon.timer < 0 {
        (*ip).xDD4_itemVar.pokemon.x64 = (*ip).xDD4_itemVar.pokemon.x64.wrapping_sub(1);
        if (*ip).xDD4_itemVar.pokemon.x64 == 0 {
            let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPokemonAttributes;
            sp10 = 1.0;
            ext::efSync_Spawn(0x471, gobj, (*(*ip).xBBC_dynamicBoneTable).bones[3], &mut sp10);
            (*ip).xDD4_itemVar.pokemon.x64 = (*attr).xC;
        }
        ext::it_8026B390(gobj);
        if (*ip).xDAC_itcmd_var0 != 0 {
            (*ip).xDAC_itcmd_var0 = 0;
            ext::Item_8026AE84(ip, 0x271F, 0x7F, 0x40);
        }
    }
}

/// 2D1204
pub unsafe extern "C" fn it_802D1204(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D1204");
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    ext::Item_8026AE84(ip, 0x2720, 0x7F, 0x40);
    if (*ip).xDC8_word.flags.x13() != 0 {
        let mut vec: Vec3 = it_803B86C8;
        ext::it_80274250(gobj, &mut vec);
    }
    ext::it_8026B3A8(gobj);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    ext::it_8026BD24(gobj);
    ext::it_8027518C(gobj);
    ext::it_80273454(gobj);
    ext::it_802756D0(gobj);
    ext::it_80275444(gobj);
    ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE);
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn itMarumine_UnkMotion6_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarumine_UnkMotion6_Anim");
    ext::it_802751D8(gobj) != 0
}

pub unsafe extern "C" fn itMarumine_UnkMotion6_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itMarumine_UnkMotion6_Phys");}

pub unsafe extern "C" fn itMarumine_UnkMotion6_Coll(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarumine_UnkMotion6_Coll");
    false
}

/// 2D1320
pub unsafe extern "C" fn it_802D1320(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D1320");
    Item_EnterAirStateWithHitlag(gobj, 0);
}

pub unsafe extern "C" fn itMarumine_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarumine_UnkMotion0_Anim");
    ext::it_80279FF8(gobj);
    false
}

pub unsafe extern "C" fn itMarumine_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itMarumine_UnkMotion0_Phys");
    if ext::it_8027A09C(gobj) != 0 {
        let ip = GET_ITEM(gobj);
        ext::Item_80268E5C(gobj, 1, ITEM_UNK_0x1);
        Item_SetEffectHitlagCallbacks(ip);
    }
}

pub unsafe extern "C" fn itMarumine_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarumine_UnkMotion0_Coll");
    ext::it_8027A118(gobj, Some(it_802D0AAC)) != 0
}
