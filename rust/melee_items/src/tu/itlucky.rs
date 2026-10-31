//! Native port of src/melee/it/kinds/itlucky.c: the Lucky Pokemon item (state machine that lays eggs through the
//! accessory callback `it_802D52E4`, the egg spawn `it_802D5710`) and the Lucky egg item (`itLucky_Logic44_*`).
//! Callees in other TUs go through `ext` (hooks; unset panics "TODO"). Checked against the C (the whole TU,
//! shipping contraction) by tests/tu_itlucky.rs. Float math: `a * (2 * (r - 0.5))` and compares only, no
//! `a*b+c` site to fuse.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, clippy::missing_safety_doc)]

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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efLib_PauseAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_PauseAll(a0) };
    fn efLib_ResumeAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_ResumeAll(a0) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BDB4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BDB4(a0) };
    fn it_8026D3CC() -> i32 => unsafe { crate::tu::itspawn::it_8026D3CC() as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80273670(a0: *mut HSD_GObj, a1: i32, a2: f32) => unsafe { crate::tu::it_2725::it_80273670(a0, a1, a2) };
    fn it_80274ED8() => unsafe { crate::tu::it_2725::it_80274ED8() };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_802754BC(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802754bc(&mut *((*a0).user_data as *mut Item)) };
    fn it_802754D4(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802754d4(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756E0(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_802756e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_80279D38(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279D38(a0) };
    fn it_80279FF8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279FF8(a0) };
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027A09C(a0) as _ };
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::it_279C::it_8027A118(a0, a1) as _ };
    fn it_80288C88(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: f32) -> *mut HSD_GObj => unsafe { crate::tu::itegg::it_80288C88(a0 as _, a1, a2, a3) as _ };
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
crate::inl_it_enter_state_hitlag!();

pub static mut it_803F8190: [ItemStateTable; 7] = [
    ItemStateTable { anim_id: -1, animated: Some(it_802D55DC), physics_updated: Some(it_802D5600), collided: Some(it_802D5620) },
    ItemStateTable { anim_id: -1, animated: Some(itLucky_UnkMotion5_Anim), physics_updated: Some(itLucky_UnkMotion5_Phys), collided: Some(itLucky_UnkMotion5_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(it_802D53AC), physics_updated: Some(it_802D53F0), collided: Some(it_802D53F4) },
    ItemStateTable { anim_id: 2, animated: Some(it_802D53AC), physics_updated: Some(it_802D53F0), collided: Some(it_802D53F4) },
    ItemStateTable { anim_id: 3, animated: Some(it_802D53AC), physics_updated: Some(it_802D53F0), collided: Some(it_802D53F4) },
    ItemStateTable { anim_id: 0, animated: Some(itLucky_UnkMotion5_Anim), physics_updated: Some(itLucky_UnkMotion5_Phys), collided: Some(itLucky_UnkMotion5_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itLucky_UnkMotion6_Anim), physics_updated: Some(itLucky_UnkMotion6_Phys), collided: None },
];

pub static mut it_803F8200: [ItemStateTable; 5] = [
    ItemStateTable { anim_id: -1, animated: Some(it_802D5884), physics_updated: Some(it_802D58BC), collided: Some(it_802D58C0) },
    ItemStateTable { anim_id: -1, animated: Some(itLucky_UnkMotion3_Anim), physics_updated: Some(itLucky_UnkMotion3_Phys), collided: Some(itLucky_UnkMotion3_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itLucky_UnkMotion2_Anim), physics_updated: Some(itLucky_UnkMotion2_Phys), collided: None },
    ItemStateTable { anim_id: -1, animated: Some(itLucky_UnkMotion3_Anim), physics_updated: Some(itLucky_UnkMotion3_Phys), collided: Some(itLucky_UnkMotion3_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(it_802D5A2C), physics_updated: Some(it_802D5A64), collided: Some(it_802D5A68) },
];

pub unsafe fn it_802D5050(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D5050");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itLuckyAttributes;
    (*ip).facing_dir = 0.0;
    (*ip).xDD4_itemVar.lucky.x60 = 2;
    (*ip).xDD4_itemVar.lucky.x64 = (*attr).x10 + ext::HSD_Randi((*attr).x14);
    ext::it_80279CDC(gobj, (*attr).x0);
    (*ip).xDD4_itemVar.lucky.x68 = ext::it_8026D3CC() as _;
    ext::Item_8026AE84(ip, 0x273B, 0x7F, 0x40);
    (*ip).xDAC_itcmd_var0 = 0;
    it_802D5560(gobj);
}

pub unsafe extern "C" fn it_802D50F0(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802D50F0");
    ext::it_80273454(gobj);
    it_802D533C(gobj);
}

pub unsafe fn it_802D5124(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D5124");
    let ip = GET_ITEM(gobj);
    let mut result = false;
    match (*ip).xDD4_itemVar.lucky.x60 {
        2 => {
            (*ip).xDD4_itemVar.lucky.x60 = 3;
        }
        3 => {
            (*ip).xDD4_itemVar.lucky.x64 -= 1;
            if (*ip).xDD4_itemVar.lucky.x64 != 0 {
                (*ip).xDD4_itemVar.lucky.x60 = 3;
            } else {
                (*ip).xDD4_itemVar.lucky.x60 = 4;
            }
        }
        4 => {
            result = true;
        }
        _ => {}
    }
    if !result {
        it_802D533C(gobj);
    }
    result as i32
}

pub unsafe fn it_802D51C8(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802D51C8");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itLuckyAttributes;
    let mut vel: Vec3 = core::mem::zeroed();
    let mut pos: Vec3 = core::mem::zeroed();
    let mut randi: i32 = 0;

    vel.x = (*attr).x4 * (2.0 * (ext::HSD_Randf() - 0.5));
    vel.y = (*attr).x8;
    vel.z = 0.0;
    let facing: f32 = if vel.x >= 0.0 { 1.0 } else { -1.0 };
    ext::lb_8000B1CC((*(*ip).xBBC_dynamicBoneTable).bones[32], null_mut(), &mut pos);
    pos.z = 0.0;
    if (*ip).xDD4_itemVar.lucky.x68 != 0 {
        randi = ext::HSD_Randi((*attr).x18);
    }
    let spawned = if randi != 0 {
        it_802D5710(gobj, &mut pos, &mut vel, facing)
    } else {
        ext::it_80288C88(gobj, &mut pos, &mut vel, facing)
    };
    if !spawned.is_null() {
        ext::it_80274ED8();
    }
}

pub unsafe extern "C" fn it_802D52E4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D52E4");
    let ip = GET_ITEM(gobj);
    if (*ip).xDAC_itcmd_var0 != 0 {
        it_802D51C8(gobj);
        ext::Item_8026AE84(ip, 0x273C, 0x7F, 0x40);
        (*ip).xDAC_itcmd_var0 = 0;
    }
}

pub unsafe fn it_802D533C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D533C");
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, (*ip).xDD4_itemVar.lucky.x60, ITEM_ANIM_UPDATE as u32);
    it_802D533C_inline(gobj);
    (*ip).on_accessory = Some(it_802D52E4);
}

pub unsafe fn it_802D533C_inline(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D533C_inline");
    let ip = GET_ITEM(gobj);
    (*ip).xDAC_itcmd_var0 = 0;
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn it_802D53AC(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D53AC");
    if ext::it_80272C6C(gobj) == 0 {
        return it_802D5124(gobj);
    }
    0
}

pub unsafe extern "C" fn it_802D53F0(_gobj: *mut Item_GObj) { melee_core::ct!("it_802D53F0");}

pub unsafe extern "C" fn it_802D53F4(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D53F4");
    ext::it_8026D62C(gobj, Some(it_802D546C));
    0
}

pub unsafe extern "C" fn it_802D5420(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802D5420");
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
    ext::it_802756E0(gobj);
    ext::it_802754D4(gobj);
    ext::it_802754BC(gobj);
}

pub unsafe extern "C" fn it_802D546C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D546C");
    ext::Item_80268E5C(gobj, 5, ITEM_UNK_0x1 as u32);
    it_802D546C_inline(gobj);
}

pub unsafe fn it_802D546C_inline(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D546C_inline");
    let ip = GET_ITEM(gobj);
    (*ip).xDAC_itcmd_var0 = 0;
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn itLucky_UnkMotion5_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLucky_UnkMotion5_Anim");
    0
}

pub unsafe extern "C" fn itLucky_UnkMotion5_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLucky_UnkMotion5_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itLucky_UnkMotion5_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLucky_UnkMotion5_Coll");
    ext::it_8026E15C(gobj, Some(it_802D50F0));
    0
}

pub unsafe extern "C" fn itLucky_UnkMotion6_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLucky_UnkMotion6_Anim");
    0
}

pub unsafe extern "C" fn itLucky_UnkMotion6_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLucky_UnkMotion6_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe fn it_802D5560(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802D5560");
    Item_EnterStateWithEffectHitlag(gobj, 0);
    ext::it_80273670(gobj, 0, 0.0);
    ext::it_8026BDB4(gobj);
}

pub unsafe extern "C" fn it_802D55DC(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D55DC");
    ext::it_80279FF8(gobj);
    0
}

pub unsafe extern "C" fn it_802D5600(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D5600");
    ext::it_8027A09C(gobj);
}

pub unsafe extern "C" fn it_802D5620(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D5620");
    ext::it_8027A118(gobj, Some(it_802D5420))
}

pub unsafe fn it_802D5648(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D5648");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itLuckyAttributes;
    if (*ip).xC9C as f32 >= (*attrs).xC {
        ext::it_80279D38(gobj);
        it_802D5648_inline(gobj);
    }
    0
}

pub unsafe fn it_802D5648_inline(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D5648_inline");
    let ip = GET_ITEM(gobj);
    (*ip).facing_dir = 0.0;
    ext::it_802762BC(ip);
    ext::it_802756D0(gobj);
    ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE as u32);
    ext::it_80273670(gobj, 0, 0.0);
}

pub unsafe fn it_802D56F0(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D56F0");
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe fn it_802D5710_inline(gobj: *mut Item_GObj, pos: *mut Vec3, vel: *mut Vec3, facing: f32) -> *mut Item_GObj {
    melee_core::ct!("it_802D5710_inline");
    let ip = (*gobj).user_data as *mut Item;
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = It_Kind_Lucky_Egg as _;
    spawn.prev_pos = *pos;
    spawn.prev_pos.z = 0.0;
    spawn.pos = spawn.prev_pos;
    spawn.facing_dir = facing;
    spawn.x3C_damage = 0;
    spawn.vel = *vel;
    spawn.x0_parent_gobj = (*ip).owner;
    spawn.x4_parent_gobj2 = gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;
    ext::Item_80268B18(&mut spawn)
}

pub unsafe fn it_802D5710(gobj: *mut Item_GObj, pos: *mut Vec3, vel: *mut Vec3, facing: f32) -> *mut Item_GObj {
    melee_core::ct!("it_802D5710");
    let mut result: *mut Item_GObj = null_mut();
    if !gobj.is_null() {
        result = it_802D5710_inline(gobj, pos, vel, facing);
    }
    result
}

pub unsafe fn itLucky_Logic44_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itLucky_Logic44_Spawned");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itLuckyEggAttributes;
    ext::it_80275158(gobj, (*attr).x0);
    (*ip).xDD4_itemVar.lucky.x60 = (*attr).x4;
    it_802D58EC(gobj);
}

pub unsafe extern "C" fn it_802D582C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D582C");
    let ip = GET_ITEM(gobj);
    ext::it_8026B390(gobj);
    itResetVelocity(ip);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn it_802D5884(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D5884");
    let ip = GET_ITEM(gobj);
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        return 1;
    }
    0
}

pub unsafe extern "C" fn it_802D58BC(_gobj: *mut Item_GObj) { melee_core::ct!("it_802D58BC");}

pub unsafe extern "C" fn it_802D58C0(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D58C0");
    ext::it_8026D62C(gobj, Some(it_802D58EC));
    0
}

pub unsafe extern "C" fn it_802D58EC(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802D58EC");
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itLucky_UnkMotion3_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLucky_UnkMotion3_Anim");
    it_802D5884(gobj)
}

pub unsafe extern "C" fn itLucky_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLucky_UnkMotion3_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itLucky_UnkMotion3_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLucky_UnkMotion3_Coll");
    ext::it_8026E15C(gobj, Some(it_802D582C));
    0
}

pub unsafe fn itLucky_Logic44_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itLucky_Logic44_PickedUp");
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itLucky_UnkMotion2_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLucky_UnkMotion2_Anim");
    0
}

pub unsafe extern "C" fn itLucky_UnkMotion2_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itLucky_UnkMotion2_Phys");}

pub unsafe fn itLucky_Logic44_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itLucky_Logic44_Dropped");
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE as u32);
}

pub unsafe fn itLucky_Logic44_EnteredAir(gobj: *mut Item_GObj) {
    melee_core::ct!("itLucky_Logic44_EnteredAir");
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn it_802D5A2C(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D5A2C");
    it_802D5884(gobj)
}

pub unsafe extern "C" fn it_802D5A64(_gobj: *mut Item_GObj) { melee_core::ct!("it_802D5A64");}

pub unsafe extern "C" fn it_802D5A68(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D5A68");
    ext::it_8026E8C4(gobj, Some(it_802D582C), Some(it_802D58EC));
    0
}

pub unsafe fn itLucky_Logic44_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itLucky_Logic44_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}
