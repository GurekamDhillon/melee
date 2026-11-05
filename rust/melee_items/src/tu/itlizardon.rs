//! Native port of src/melee/it/kinds/itlizardon.c: the Charizard Pokemon item (state machine with the fire
//! breath accessory callback `it_802CBD24`, spawn of the flame items `it_802CC1CC`, the four flame logic
//! spawns and the flame items' states). Callees in other TUs go through `ext` (hooks; unset panics "TODO").
//! Checked against the C (the whole TU, shipping contraction) by tests/tu_itlizardon.rs.
//!
//! Float math: no `a*b+c` site (products of sums and differences only), so nothing to fuse; `x - int` is
//! `x - (f32)int`. `efSync_Spawn` is `ext::efSync_Spawn_N(gfx, gobj, &[args])`. `HSD_Randi` is the ext hook.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::itkinds7::bisect_value;
#[allow(unused_imports)]
use crate::layout::itEgg_ItemVars;
#[allow(unused_imports)]
use crate::layout::itPeachTurnipAttributes;
#[allow(unused_imports)]
use crate::layout::itTaruAttributes;
#[allow(unused_imports)]
use crate::layout::it_2E5A_Attrs;
#[allow(unused_imports)]
use crate::layout::ItemPickTable;
#[allow(unused_imports)]
use melee_core::fp::frsqrte;
#[allow(unused_imports)]
use melee_runtime::random::HsdRand;
#[allow(unused_imports)]
use crate::itkinds9::{lizardon_x70};
#[allow(unused_imports)]
use crate::helpers::it_80275158;
#[allow(unused_imports)]
use crate::helpers::it_rshell_clamp_vel;
#[allow(unused_imports)]
use crate::helpers::itRShell_Attrs;
#[allow(unused_imports)]
use crate::item::decelerate_item_x;
#[allow(unused_imports)]
use crate::item::it_802762bc;
#[allow(unused_imports)]
use crate::itlib::it_80275444;
#[allow(unused_imports)]
use crate::itlib::it_802756d0;
#[allow(unused_imports)]
use crate::itlib::it_8027572c;
#[allow(unused_imports)]
use crate::itlib3::it_80272860;
#[allow(unused_imports)]
use crate::itlib3::it_802756e0;
#[allow(unused_imports)]
use crate::itlib3::it_coll_bounce_off_victim;
#[allow(unused_imports)]
use crate::layout::itClimbersIceAttributes;
#[allow(unused_imports)]
use crate::layout::itCoinAttributes;
#[allow(unused_imports)]
use crate::layout::itDoseiAttributes;
#[allow(unused_imports)]
use crate::layout::itFireAttributes;
#[allow(unused_imports)]
use crate::layout::itFlipper_DatAttrs;
#[allow(unused_imports)]
use crate::layout::itFreezerAttributes;
#[allow(unused_imports)]
use crate::layout::itGamewatchchefAttributes;
#[allow(unused_imports)]
use crate::layout::itHammerheadAttributes;
#[allow(unused_imports)]
use crate::layout::itHitodemanAttributes;
#[allow(unused_imports)]
use crate::layout::itHououAttr;
#[allow(unused_imports)]
use crate::layout::itLeadeadAttributes;
#[allow(unused_imports)]
use crate::layout::itLinkArrowAttributes;
#[allow(unused_imports)]
use crate::layout::itLinkBombAttributes;
#[allow(unused_imports)]
use crate::layout::itLizardonAttributes;
#[allow(unused_imports)]
use crate::layout::itMatadogasAttributes;
#[allow(unused_imports)]
use crate::layout::itPeachToadSporeAttributes;
#[allow(unused_imports)]
use crate::layout::itSamusMissileAttributes;
#[allow(unused_imports)]
use crate::layout::itsonansAttributes;
#[allow(unused_imports)]
use crate::layout::HeartContainerAttr;
#[allow(unused_imports)]
use crate::layout::ItemAttr;
#[allow(unused_imports)]
use crate::layout::KinokoAttrs;
#[allow(unused_imports)]
use crate::layout::MewVars;
#[allow(unused_imports)]
use crate::layout::Vec as GcVec;
#[allow(unused_imports)]
use crate::layout::GA_Air;
#[allow(unused_imports)]
use crate::layout::GA_Ground;
#[allow(unused_imports)]
use crate::layout::It_Kind_Lugia_Aeroblast;
#[allow(unused_imports)]
use crate::layout::It_Kind_Lugia_Aeroblast2;
#[allow(unused_imports)]
use crate::layout::It_Kind_Lugia_Aeroblast3;
#[allow(unused_imports)]
use crate::layout::It_Kind_Matadogas_Gas1;
#[allow(unused_imports)]
use crate::layout::It_Kind_Matadogas_Gas2;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use std::ptr::null_mut as null_mut_itkinds;
#[allow(unused_imports)]
use crate::itkinds::{sa};
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
use melee_runtime::gekko::{cosf, sinf};

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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efLib_PauseAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_PauseAll(a0) };
    fn efLib_ResumeAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_ResumeAll(a0) };
    fn efSync_Spawn_N(a0: i32, a1: *mut HSD_GObj, a2: &[*mut c_void]) -> *mut c_void;
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB88(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB88(a0, a1 as _) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DFB0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DFB0(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80273670(a0: *mut HSD_GObj, a1: i32, a2: f32) => unsafe { crate::tu::it_2725::it_80273670(a0, a1, a2) };
    fn it_80274740(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274740(a0) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_80279FF8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279FF8(a0) };
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027A09C(a0) as _ };
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::it_279C::it_8027A118(a0, a1) as _ };
    fn it_8027ADEC(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj, a3: f32) => unsafe { crate::tu::it_279C::it_8027ADEC(a0, a1, a2, a3) };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
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

crate::inl_it_core!();
crate::inl_it_hitlag_cb!();
crate::inl_it_enter_air_hitlag_desc!();
crate::inl_it_tick!();

#[inline(always)]
unsafe fn attrs_of(ip: *mut Item) -> *mut itLizardonAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itLizardonAttributes
}

pub static mut it_803F7BC0: [ItemStateTable; 4] = [
    ItemStateTable { anim_id: 0, animated: Some(it_802CC160), physics_updated: Some(it_802CC184), collided: Some(it_802CC1A4) },
    ItemStateTable { anim_id: 1, animated: Some(itLizardon_UnkMotion1_Anim), physics_updated: Some(itLizardon_UnkMotion1_Phys), collided: Some(itLizardon_UnkMotion1_Coll) },
    ItemStateTable { anim_id: 2, animated: Some(itLizardon_UnkMotion2_Anim), physics_updated: Some(itLizardon_UnkMotion2_Phys), collided: Some(itLizardon_UnkMotion2_Coll) },
    ItemStateTable { anim_id: 3, animated: Some(itLizardon_UnkMotion3_Anim), physics_updated: Some(itLizardon_UnkMotion3_Phys), collided: Some(itLizardon_UnkMotion3_Coll) },
];

pub static mut it_803F7C00: [ItemStateTable; 1] = [
    ItemStateTable { anim_id: 0, animated: Some(it_802CC650), physics_updated: Some(it_802CC684), collided: Some(it_802CC6C4) },
];

pub unsafe fn it_802CB8AC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CB8AC");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    (*ip).xDAC_itcmd_var0 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDB4_itcmd_var2 = 0;
    (*ip).xDD4_itemVar.lizardon.x60 = -1;
    (*ip).xDD4_itemVar.lizardon.x64 = 0;
    (*ip).xDD4_itemVar.lizardon.x68 = 0.0;
    (*ip).xDD4_itemVar.lizardon.x6C = 0;
    it_802CC0EC(gobj);
    ext::it_80279CDC(gobj, (*attrs).x0);
    ext::Item_8026AE84(ip, 0x2719, 0x7F, 0x40);
}

pub unsafe extern "C" fn it_802CB93C(_gobj: *mut HSD_GObj) { melee_core::ct!("it_802CB93C");}

pub unsafe fn it_802CB940(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CB940");
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe extern "C" fn it_802CB960(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CB960");
    ext::it_80273454(gobj);
    it_802CB994(gobj);
}

pub unsafe fn it_802CB994(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CB994");
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn itLizardon_UnkMotion1_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLizardon_UnkMotion1_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        it_802CBAA8(gobj);
    }
    0
}

pub unsafe extern "C" fn itLizardon_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLizardon_UnkMotion1_Phys");
    let ip = GET_ITEM(gobj);

    if (*ip).ground_or_air == GA_Air {
        let ip_attr = (*ip).xCC_item_attr;
        ext::it_80272860(gobj, (*ip_attr).x10_fall_speed, (*ip_attr).x14_fall_speed_max);
    }
}

pub unsafe extern "C" fn itLizardon_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLizardon_UnkMotion1_Coll");
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Air {
        ext::it_8026E15C(gobj, Some(it_802CB93C));
    } else {
        ext::it_8026D62C(gobj, Some(it_802CB93C));
    }
    0
}

pub unsafe fn it_802CBAA8_inline(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CBAA8_inline");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    (*ip).xDD4_itemVar.lizardon.x70 = (*attrs).x28 + ext::HSD_Randi((*attrs).x2C - (*attrs).x28);
}

pub unsafe fn it_802CBAA8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CBAA8");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);

    if (*ip).xDD4_itemVar.lizardon.x60 == -1 {
        (*ip).xDD4_itemVar.lizardon.x60 = *(&raw const (*attrs).x4 as *const i32);
    }
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as u32);
    it_802CBAA8_inline(gobj);
    (*ip).xDD4_itemVar.lizardon.x74[0] = (*attrs).x18[0];
    (*ip).xDD4_itemVar.lizardon.x74[1] = (*attrs).x18[1];
    (*ip).xDD4_itemVar.lizardon.x74[2] = (*attrs).x18[2];
    (*ip).xDD4_itemVar.lizardon.x74[3] = (*attrs).x18[3];
    Item_SetEffectHitlagCallbacks(ip);
    (*ip).on_accessory = Some(it_802CBD24);
}

pub unsafe extern "C" fn itLizardon_UnkMotion2_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLizardon_UnkMotion2_Anim");
    let ip = GET_ITEM(gobj);
    if ext::it_80272C6C(gobj) == 0 {
        if (*ip).xDD4_itemVar.lizardon.x60 <= 0 {
            (*ip).xDD4_itemVar.lizardon.x60 = 0;
            it_802CBFE4(gobj);
        } else {
            (*ip).xDD4_itemVar.lizardon.x60 -= 1;
            it_802CBAA8(gobj);
        }
    }
    0
}

pub unsafe extern "C" fn itLizardon_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLizardon_UnkMotion2_Phys");
    itLizardon_UnkMotion1_Phys(gobj);
}

pub unsafe extern "C" fn itLizardon_UnkMotion2_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLizardon_UnkMotion2_Coll");
    itLizardon_UnkMotion1_Coll(gobj)
}

pub unsafe fn it_802CBD24_inline(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CBD24_inline");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    (*ip).xDD4_itemVar.lizardon.x70 = (*attrs).x28 + ext::HSD_Randi((*attrs).x2C - (*attrs).x28);
}

pub unsafe extern "C" fn it_802CBD24(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CBD24");
    let mut bone_pos: Vec3 = core::mem::zeroed();
    let mut scale: f32;
    let mut facing: f32;
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);

    if (*ip).xDB0_itcmd_var1 != 0 {
        (*ip).xDD4_itemVar.lizardon.x64 ^= 1;
        (*ip).xDD4_itemVar.lizardon.x68 = (*attrs).x14 as f32;
        (*ip).xDB0_itcmd_var1 = 0;
    }
    if (*ip).xDAC_itcmd_var0 != 0 {
        let attrs2 = attrs_of(ip);
        scale = 1.0;
        ext::lb_8000B1CC((*(*ip).xBBC_dynamicBoneTable).bones[0x32], null_mut(), &mut bone_pos);
        if (*ip).xDD4_itemVar.lizardon.x64 != 0 {
            facing = -1.0;
        } else {
            facing = 1.0;
        }
        {
            let v = (*ip).xDD4_itemVar.lizardon.x68 - 1.0;
            (*ip).xDD4_itemVar.lizardon.x68 = v;
            if v == 0.0 {
                (*ip).xDD4_itemVar.lizardon.x68 = (*attrs2).x14 as f32;
                it_802CC1CC(gobj, ext::HSD_Randi(4) + 0xC3);
            }
        }
        {
            let v = (*ip).xDD4_itemVar.lizardon.x70 - 1;
            (*ip).xDD4_itemVar.lizardon.x70 = v;
            if v < 0 {
                ext::it_8027ADEC(0x452, gobj, (*(*ip).xBBC_dynamicBoneTable).bones[0x31], 1.4);
                it_802CBD24_inline(gobj);
            }
        }
        {
            let v = (*ip).xDD4_itemVar.lizardon.x74[0] - 1;
            (*ip).xDD4_itemVar.lizardon.x74[0] = v;
            if v < 0 {
                scale = (scale / 1.4) * (*(*ip).xCC_item_attr).x60_scale;
                ext::efSync_Spawn_N(0x453, gobj, &[(*(*ip).xBBC_dynamicBoneTable).bones[0x32] as *mut c_void, &raw mut facing as *mut c_void, &raw mut scale as *mut c_void]);
                (*ip).xDD4_itemVar.lizardon.x74[0] = (*attrs2).x18[0];
            }
        }
        {
            let v = (*ip).xDD4_itemVar.lizardon.x74[1] - 1;
            (*ip).xDD4_itemVar.lizardon.x74[1] = v;
            if v < 0 {
                scale = (scale / 1.4) * (*(*ip).xCC_item_attr).x60_scale;
                ext::efSync_Spawn_N(0x454, gobj, &[(*(*ip).xBBC_dynamicBoneTable).bones[0x32] as *mut c_void, &raw mut facing as *mut c_void, &raw mut scale as *mut c_void]);
                (*ip).xDD4_itemVar.lizardon.x74[1] = (*attrs2).x18[1];
            }
        }
        {
            let v = (*ip).xDD4_itemVar.lizardon.x74[2] - 1;
            (*ip).xDD4_itemVar.lizardon.x74[2] = v;
            if v < 0 {
                scale = (scale / 1.4) * (*(*ip).xCC_item_attr).x60_scale;
                ext::efSync_Spawn_N(0x455, gobj, &[(*(*ip).xBBC_dynamicBoneTable).bones[0x32] as *mut c_void, &raw mut facing as *mut c_void, &raw mut scale as *mut c_void]);
                (*ip).xDD4_itemVar.lizardon.x74[2] = (*attrs2).x18[2];
            }
        }
        {
            let v = (*ip).xDD4_itemVar.lizardon.x74[3] - 1;
            (*ip).xDD4_itemVar.lizardon.x74[3] = v;
            if v < 0 {
                scale = (scale / 1.4) * (*(*ip).xCC_item_attr).x60_scale;
                ext::efSync_Spawn_N(0x456, gobj, &[(*(*ip).xBBC_dynamicBoneTable).bones[0x32] as *mut c_void, &raw mut facing as *mut c_void, &raw mut scale as *mut c_void]);
                (*ip).xDD4_itemVar.lizardon.x74[3] = (*attrs2).x18[3];
            }
        }
    }
}

pub unsafe fn it_802CBFE4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CBFE4");
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE as u32);
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn itLizardon_UnkMotion3_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLizardon_UnkMotion3_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        return 1;
    }
    0
}

pub unsafe extern "C" fn itLizardon_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLizardon_UnkMotion3_Phys");
    itLizardon_UnkMotion1_Phys(gobj);
}

pub unsafe extern "C" fn itLizardon_UnkMotion3_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLizardon_UnkMotion3_Coll");
    itLizardon_UnkMotion1_Coll(gobj)
}

pub unsafe fn it_802CC0EC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CC0EC");
    Item_EnterAirStateWithHitlagAndStateDesc(gobj);
}

pub unsafe extern "C" fn it_802CC160(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802CC160");
    ext::it_80279FF8(gobj);
    0
}

pub unsafe extern "C" fn it_802CC184(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CC184");
    ext::it_8027A09C(gobj);
}

pub unsafe extern "C" fn it_802CC1A4(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802CC1A4");
    ext::it_8027A118(gobj, Some(it_802CB960))
}

pub unsafe fn it_802CC1CC(gobj: *mut Item_GObj, kind: ItemKind) {
    melee_core::ct!("it_802CC1CC");
    let mut spawn: SpawnItem = core::mem::zeroed();
    let mut bone_pos: Vec3 = core::mem::zeroed();
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);

    ext::lb_8000B1CC((*(*ip).xBBC_dynamicBoneTable).bones[0x32], null_mut(), &mut bone_pos);
    spawn.prev_pos = bone_pos;
    ext::it_8026BB88(gobj, &mut spawn.pos);
    spawn.x3C_damage = 0;
    let randi = ext::HSD_Randi(melee_core::ppc::fctiwz_f32((*attrs).x8 - (*attrs).xC));
    let angle = 0.017453292_f32 * ((*attrs).x8 - randi as f32);
    if (*ip).xDD4_itemVar.lizardon.x64 != 0 {
        spawn.vel.x = -1.0 * ((*attrs).x10 * cosf(angle));
        spawn.facing_dir = -1.0;
    } else {
        spawn.vel.x = (*attrs).x10 * cosf(angle);
        spawn.facing_dir = 1.0;
    }
    spawn.vel.y = (*attrs).x10 * sinf(angle);
    spawn.vel.z = 0.0;
    spawn.kind = kind;
    spawn.x0_parent_gobj = (*ip).owner;
    spawn.x4_parent_gobj2 = gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;
    if !ext::Item_80268B18(&mut spawn).is_null() {
        (*ip).xDD4_itemVar.lizardon.x6C ^= 1;
        if (*ip).xDD4_itemVar.lizardon.x6C != 0 {
            match ext::HSD_Randi(3) {
                0 => {
                    ext::Item_8026AE84(ip, 0x2716, 0x7F, 0x40);
                    return;
                }
                1 => {
                    ext::Item_8026AE84(ip, 0x2717, 0x7F, 0x40);
                    return;
                }
                2 => {
                    ext::Item_8026AE84(ip, 0x2718, 0x7F, 0x40);
                }
                _ => {}
            }
        }
    }
}

pub unsafe fn itLizardon_Logic34_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itLizardon_Logic34_Spawned");
    let ip = (*gobj).user_data as *mut Item;
    let hsd_obj = (*gobj).hsd_obj;
    let f = 0.71428573_f32;
    let mut f2 = f * (*(*ip).xCC_item_attr).x60_scale;
    ext::efSync_Spawn_N(0x44E, gobj, &[hsd_obj, &raw mut (*ip).facing_dir as *mut c_void, &raw mut f2 as *mut c_void]);
    it_802CC5D4(gobj);
}

pub unsafe fn itLizardon_Logic35_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itLizardon_Logic35_Spawned");
    let ip = (*gobj).user_data as *mut Item;
    let hsd_obj = (*gobj).hsd_obj;
    let f = 0.71428573_f32;
    let mut f2 = f * (*(*ip).xCC_item_attr).x60_scale;
    ext::efSync_Spawn_N(0x44F, gobj, &[hsd_obj, &raw mut (*ip).facing_dir as *mut c_void, &raw mut f2 as *mut c_void]);
    it_802CC5D4(gobj);
}

pub unsafe fn itLizardon_Logic36_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itLizardon_Logic36_Spawned");
    let ip = (*gobj).user_data as *mut Item;
    let hsd_obj = (*gobj).hsd_obj;
    let f = 0.71428573_f32;
    let mut f2 = f * (*(*ip).xCC_item_attr).x60_scale;
    ext::efSync_Spawn_N(0x450, gobj, &[hsd_obj, &raw mut (*ip).facing_dir as *mut c_void, &raw mut f2 as *mut c_void]);
    it_802CC5D4(gobj);
}

pub unsafe fn itLizardon_Logic37_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itLizardon_Logic37_Spawned");
    let ip = (*gobj).user_data as *mut Item;
    let hsd_obj = (*gobj).hsd_obj;
    let f = 0.71428573_f32;
    let mut f2 = f * (*(*ip).xCC_item_attr).x60_scale;
    ext::efSync_Spawn_N(0x451, gobj, &[hsd_obj, &raw mut (*ip).facing_dir as *mut c_void, &raw mut f2 as *mut c_void]);
    it_802CC5D4(gobj);
}

pub unsafe fn itLizardon_Logic37_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itLizardon_Logic37_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe fn itLizardon_Logic37_Reflected(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLizardon_Logic37_Reflected");
    ext::it_80273030(gobj)
}

pub unsafe fn itLizardon_Logic37_HitShield(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLizardon_Logic37_HitShield");
    1
}

pub unsafe fn itLizardon_Logic37_Absorbed(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLizardon_Logic37_Absorbed");
    1
}

pub unsafe fn it_802CC5D4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CC5D4");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    ext::it_80275158(gobj, (*attrs).x0);
    ext::it_80274740(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
    Item_SetEffectHitlagCallbacks(ip);
    ext::it_8026B3A8(gobj);
}

pub unsafe extern "C" fn it_802CC650(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802CC650");
    let ip = GET_ITEM(gobj);
    Item_TickLifetime(ip) as i32
}

pub unsafe extern "C" fn it_802CC684(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CC684");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);

    (*ip).x40_vel.x *= (*attrs).x4;
    (*ip).x40_vel.y *= (*attrs).x4;
    (*ip).x40_vel.z *= (*attrs).x4;
}

pub unsafe extern "C" fn it_802CC6C4(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802CC6C4");
    ext::it_8026DFB0(gobj)
}

// ---- folded from itfwd.rs (consolidation steps 21/22) ----
/// `it_802CB940` (src/melee/it/kinds/itlizardon.c): `it_8026B894(..)`.
pub fn it_802cb940(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802cb940");
    it_8026b894(ip, ref_gobj);
}

// ---- folded from itfwd8.rs (consolidation steps 21/22) ----
/// `it_802CB93C` (src/melee/it/kinds/itlizardon.c): `empty body / return;`.
pub fn it_802cb93c(_ip: &mut Item) { melee_core::ct!("it_802cb93c");}

// ---- folded from itkinds.rs (consolidation steps 21/22) ----
/// `it_802CC684` (itlizardon.c): scale the velocity by `attrs.x4`.
/// # Safety
/// The special attributes must be a valid `itLizardonAttributes`.
pub unsafe fn it_802cc684(ip: &mut Item) {
    melee_core::ct!("it_802cc684");
    let k = sa::<itLizardonAttributes>(ip).x4;
    ip.x40_vel.x *= k;
    ip.x40_vel.y *= k;
    ip.x40_vel.z *= k;
}

// ---- folded from itkinds9.rs (consolidation steps 21/22) ----
/// `it_802CBAA8_inline` (itlizardon.c): `x70 = x28 + HSD_Randi(x2C - x28)`.
/// # Safety
/// The item's article data and special attributes are valid.
pub unsafe fn it_802cbaa8_inline(rand: &mut HsdRand, ip: &mut Item) {
    melee_core::ct!("it_802cbaa8_inline");
    lizardon_x70(rand, ip)
}

/// `it_802CBD24_inline` (itlizardon.c): same body as `it_802CBAA8_inline`.
/// # Safety
/// As `it_802cbaa8_inline`.
pub unsafe fn it_802cbd24_inline(rand: &mut HsdRand, ip: &mut Item) {
    melee_core::ct!("it_802cbd24_inline");
    lizardon_x70(rand, ip)
}
