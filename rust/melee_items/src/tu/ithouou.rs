//! Native port of src/melee/it/kinds/ithouou.c (Ho-Oh Pokemon and its sacred fire). Callees in
//! other TUs go through `ext`. Checked against the C (whole TU, shipping contraction) by
//! tests/tu_ithouou.rs. No fused sites.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

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
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Stage_GetBlastZoneBottomOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneBottomOffset() };
    fn Stage_GetBlastZoneTopOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneTopOffset() };
    fn efLib_PauseAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_PauseAll(a0) };
    fn efLib_ResumeAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_ResumeAll(a0) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80273670(a0: *mut HSD_GObj, a1: i32, a2: f32) => unsafe { crate::tu::it_2725::it_80273670(a0, a1, a2) };
    fn it_80274740(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274740(a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_80279FF8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279FF8(a0) };
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027A09C(a0) as _ };
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::it_279C::it_8027A118(a0, a1) as _ };
    fn it_8027A160(a0: *mut HSD_JObj, a1: *mut Item) => unsafe { crate::tu::it_279C::it_8027A160(a0, a1) };
    fn it_8027A344(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_8027a344(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027ADEC(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj, a3: f32) => unsafe { crate::tu::it_279C::it_8027ADEC(a0, a1, a2, a3) };
    fn mpCheckAllRemap(a0: *mut Vec, a1: *mut i32, a2: *mut u32, a3: *mut Vec, a4: i32, a5: i32, a6: f32, a7: f32, a8: f32, a9: f32) -> i32 => unsafe { melee_world::tu::mplib::mpCheckAllRemap(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9) };
    fn mpGetSpeed(a0: i32, a1: *mut Vec, a2: *mut Vec) -> i32 => unsafe { melee_world::tu::mplib::mpGetSpeed(a0, a1, a2) };
    fn mpLib_80054ED8(a0: i32) -> i32 => unsafe { melee_world::tu::mplib::mpLib_80054ED8(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn efLib_PauseAll(a0: *mut HSD_GObj) { super::efLib_PauseAll(a0) }
        pub unsafe extern "C" fn efLib_ResumeAll(a0: *mut HSD_GObj) { super::efLib_ResumeAll(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut item: *mut Item = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

#[inline]
unsafe fn ip_of(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}
#[inline]
unsafe fn attr_of(ip: *mut Item) -> *mut itHououAttr {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itHououAttr
}
#[inline]
unsafe fn set_effect_hitlag_callbacks(ip: *mut Item) {
    (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}
/// `jobj ? jobj->child : NULL`
#[inline]
unsafe fn child_of(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    if jobj.is_null() {
        null_mut()
    } else {
        (*jobj).child
    }
}

/// `ItemStateTable it_803F7F58[6]` (3F7F58)
pub static it_803F7F58: [ItemStateTable; 6] = [
    ItemStateTable { anim_id: 0, animated: Some(it_802D2C54), physics_updated: Some(it_802D2C78), collided: Some(it_802D2D04) },
    ItemStateTable { anim_id: 1, animated: Some(itHouou_UnkMotion1_Anim), physics_updated: Some(itHouou_UnkMotion1_Phys), collided: Some(itHouou_UnkMotion1_Coll) },
    ItemStateTable { anim_id: 2, animated: Some(itHouou_UnkMotion2_Anim), physics_updated: Some(itHouou_UnkMotion2_Phys), collided: Some(itHouou_UnkMotion2_Coll) },
    ItemStateTable { anim_id: 3, animated: Some(itHouou_UnkMotion3_Anim), physics_updated: Some(itHouou_UnkMotion3_Phys), collided: Some(itHouou_UnkMotion3_Coll) },
    ItemStateTable { anim_id: 4, animated: Some(itHouou_UnkMotion4_Anim), physics_updated: Some(itHouou_UnkMotion4_Phys), collided: Some(itHouou_UnkMotion4_Coll) },
    ItemStateTable { anim_id: 5, animated: Some(itHouou_UnkMotion5_Anim), physics_updated: Some(itHouou_UnkMotion5_Phys), collided: Some(itHouou_UnkMotion5_Coll) },
];

/// `ItemStateTable it_803F7FB8[1]` (3F7FB8)
pub static it_803F7FB8: [ItemStateTable; 1] =
    [ItemStateTable { anim_id: 0, animated: Some(it_802D2F3C), physics_updated: Some(it_802D2F70), collided: Some(it_802D2FE8) }];

/// `void it_2725_Logic18_Spawned(Item_GObj*)` (2D251C)
pub unsafe extern "C" fn it_2725_Logic18_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("it_2725_Logic18_Spawned");
    let ip = ip_of(gobj);
    let attr = attr_of(ip);
    (*ip).xDD4_itemVar.houou.start_pos = (*ip).pos;
    (*ip).facing_dir = 0.0;
    (*ip).xDAC_itcmd_var0 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDD4_itemVar.houou.timer = (*attr).x14;
    (*ip).xDCC_flag.set_b3(0);
    it_802D2BE0(gobj);
    ext::it_80279CDC(gobj, (*attr).timer);
    (*ip).xDD4_itemVar.houou.vel_accum = 0.0;
}

/// `void it_802D25B8(HSD_GObj*)` (2D25B8)
pub unsafe extern "C" fn it_802D25B8(_gobj: *mut HSD_GObj) { melee_core::ct!("it_802D25B8");}

/// `void it_802D25BC(Item_GObj*, Item_GObj*)` (2D25BC)
pub unsafe extern "C" fn it_802D25BC(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D25BC");
    ext::it_8026B894(gobj, ref_gobj);
}

/// `bool itHouou_UnkMotion1_Anim(Item_GObj*)` (2D25DC)
pub unsafe extern "C" fn itHouou_UnkMotion1_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itHouou_UnkMotion1_Anim");
    let ip = ip_of(gobj);
    ext::it_8027A160(child_of(gobj), ip);
    if ext::it_80272C6C(gobj) == 0 {
        it_802D2668(gobj);
    }
    0
}

/// `void itHouou_UnkMotion1_Phys(Item_GObj*)` (2D2640)
pub unsafe extern "C" fn itHouou_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itHouou_UnkMotion1_Phys");
    ext::it_8027A344(gobj);
}

/// `bool itHouou_UnkMotion1_Coll(Item_GObj*)` (2D2660)
pub unsafe extern "C" fn itHouou_UnkMotion1_Coll(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itHouou_UnkMotion1_Coll");
    0
}

/// `void it_802D2668(Item_GObj*)` (2D2668)
pub unsafe extern "C" fn it_802D2668(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D2668");
    let ip = ip_of(gobj);
    (*ip).xDD4_itemVar.houou.vel_accum = 0.0;
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(ip);
}

/// `bool itHouou_UnkMotion2_Anim(Item_GObj*)` (2D26BC)
pub unsafe extern "C" fn itHouou_UnkMotion2_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itHouou_UnkMotion2_Anim");
    let item = ip_of(gobj);
    ext::it_8027A160(child_of(gobj), item);
    0
}

/// `void itHouou_UnkMotion2_Phys(Item_GObj*)` (2D26FC)
pub unsafe extern "C" fn itHouou_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itHouou_UnkMotion2_Phys");
    let ip = ip_of(gobj);
    let attr = attr_of(ip);

    ext::it_8027A344(gobj);

    if ext::it_80272C6C(gobj) != 0 {
        (*ip).xDD4_itemVar.houou.vel_accum += (*attr).x8;
    } else {
        (*ip).xDD4_itemVar.houou.vel_accum += (*attr).xC;
    }

    (*ip).x40_vel.y += (*ip).xDD4_itemVar.houou.vel_accum;

    if (*ip).pos.y > ext::Stage_GetBlastZoneTopOffset() {
        (*ip).x40_vel.y = 0.0;
        it_802D27B0(gobj);
    }
}

/// `bool itHouou_UnkMotion2_Coll(Item_GObj*)` (2D27A8)
pub unsafe extern "C" fn itHouou_UnkMotion2_Coll(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itHouou_UnkMotion2_Coll");
    0
}

/// `void it_802D27B0(Item_GObj*)` (2D27B0)
pub unsafe extern "C" fn it_802D27B0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D27B0");
    let ip = ip_of(gobj);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(ip);
}

/// `bool itHouou_UnkMotion3_Anim(Item_GObj*)` (2D27FC)
pub unsafe extern "C" fn itHouou_UnkMotion3_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itHouou_UnkMotion3_Anim");
    let ip = ip_of(gobj);
    ext::it_8027A160(child_of(gobj), ip);
    if ext::it_80272C6C(gobj) == 0 {
        let ip2 = ip_of(gobj);
        ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE as u32);
        set_effect_hitlag_callbacks(ip2);
    }
    0
}

/// `void itHouou_UnkMotion3_Phys(Item_GObj*)` (2D288C)
pub unsafe extern "C" fn itHouou_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itHouou_UnkMotion3_Phys");
    let ip = ip_of(gobj);
    let attr = attr_of(ip);

    ext::it_8027A344(gobj);

    let timer = (*ip).xDD4_itemVar.houou.timer;
    (*ip).xDD4_itemVar.houou.timer = timer.wrapping_sub(1);

    if timer != 0 {
        (*ip).x40_vel.z = (*attr).x10;
        return;
    }

    (*ip).x40_vel.z = 0.0;
    it_802D290C(gobj);
}

/// `bool itHouou_UnkMotion3_Coll(Item_GObj*)` (2D2904)
pub unsafe extern "C" fn itHouou_UnkMotion3_Coll(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itHouou_UnkMotion3_Coll");
    0
}

/// `void it_802D290C(Item_GObj*)` (2D290C)
pub unsafe extern "C" fn it_802D290C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D290C");
    let ip = ip_of(gobj);
    let attr = attr_of(ip);
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(ip);
    (*ip).xDD4_itemVar.houou.vel_accum = (*attr).x18;
}

/// `bool itHouou_UnkMotion4_Anim(Item_GObj*)` (2D2970)
pub unsafe extern "C" fn itHouou_UnkMotion4_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itHouou_UnkMotion4_Anim");
    let ip = ip_of(gobj);
    ext::it_8027A160(child_of(gobj), ip);
    if ext::it_80272C6C(gobj) == 0 {
        it_802D2A58(gobj);
    }
    0
}

/// `void itHouou_UnkMotion4_Phys(Item_GObj*)` (2D29D4)
pub unsafe extern "C" fn itHouou_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itHouou_UnkMotion4_Phys");
    let ip = ip_of(gobj);
    let attr = attr_of(ip);

    ext::it_8027A344(gobj);

    (*ip).xDD4_itemVar.houou.vel_accum += (*attr).x1C;
    (*ip).x40_vel.y += (*ip).xDD4_itemVar.houou.vel_accum;

    if (*ip).pos.y < (*ip).xDD4_itemVar.houou.start_pos.y - 10.0 {
        (*ip).x40_vel.y = 0.0;
    }
}

/// `bool itHouou_UnkMotion4_Coll(Item_GObj*)` (2D2A50)
pub unsafe extern "C" fn itHouou_UnkMotion4_Coll(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itHouou_UnkMotion4_Coll");
    0
}

/// `void it_802D2A58(Item_GObj*)` (2D2A58)
pub unsafe extern "C" fn it_802D2A58(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D2A58");
    let ip = ip_of(gobj);
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(ip);
    (*ip).on_accessory = Some(it_802D2B4C);
    (*ip).xDCC_flag.set_b3(1);
}

/// `bool itHouou_UnkMotion5_Anim(Item_GObj*)` (2D2AC0)
pub unsafe extern "C" fn itHouou_UnkMotion5_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itHouou_UnkMotion5_Anim");
    let ip = ip_of(gobj);
    ext::it_8027A160(child_of(gobj), ip);
    if ext::it_80272C6C(gobj) == 0 {
        return 1;
    }
    0
}

/// `void itHouou_UnkMotion5_Phys(Item_GObj*)` (2D2B24)
pub unsafe extern "C" fn itHouou_UnkMotion5_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itHouou_UnkMotion5_Phys");
    ext::it_8027A344(gobj);
}

/// `bool itHouou_UnkMotion5_Coll(Item_GObj*)` (2D2B44)
pub unsafe extern "C" fn itHouou_UnkMotion5_Coll(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itHouou_UnkMotion5_Coll");
    0
}

/// `void it_802D2B4C(Item_GObj*)` (2D2B4C)
pub unsafe extern "C" fn it_802D2B4C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D2B4C");
    let ip = ip_of(gobj);

    if (*ip).xDAC_itcmd_var0 != 0 {
        (*ip).xDAC_itcmd_var0 = 0;
        ext::it_8027ADEC(0x46F, gobj, (*(*ip).xBBC_dynamicBoneTable).bones[16], 1.3);
        ext::it_8027ADEC(0x46F, gobj, (*(*ip).xBBC_dynamicBoneTable).bones[17], 1.3);
    }

    if (*ip).xDB0_itcmd_var1 != 0 {
        (*ip).xDB0_itcmd_var1 = 0;
        it_802D2D2C(gobj);
    }
}

/// `void it_802D2BE0(Item_GObj*)` (2D2BE0): `Item_EnterAirStateWithHitlagAndStateDesc`.
pub unsafe extern "C" fn it_802D2BE0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D2BE0");
    let item = ip_of(gobj);
    ext::it_802762BC(item);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(item);
    ext::it_80273670(gobj, 0, 0.0);
}

/// `bool it_802D2C54(Item_GObj*)` (2D2C54)
pub unsafe extern "C" fn it_802D2C54(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D2C54");
    ext::it_80279FF8(gobj);
    0
}

/// `void it_802D2C78(Item_GObj*)` (2D2C78)
pub unsafe extern "C" fn it_802D2C78(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D2C78");
    if ext::it_8027A09C(gobj) != 0 {
        let ip = ip_of(gobj);
        ext::it_80273454(gobj);
        let ip2 = ip_of(gobj);
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
        set_effect_hitlag_callbacks(ip2);
        (*ip).xDD1_flag.anon_0.set_b1(1);
    }
}

/// `bool it_802D2D04(Item_GObj*)` (2D2D04)
pub unsafe extern "C" fn it_802D2D04(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D2D04");
    ext::it_8027A118(gobj, Some(it_802D25B8))
}

/// `void it_802D2D2C(Item_GObj*)` (2D2D2C): drop the sacred fire where the line below the start is.
pub unsafe extern "C" fn it_802D2D2C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D2D2C");
    let mut spawn: SpawnItem = core::mem::zeroed();
    let mut hit_pos = Vec { x: 0.0, y: 0.0, z: 0.0 };
    let mut line_id: i32 = 0;
    let ip = ip_of(gobj);

    let tmp = (*ip).xDD4_itemVar.houou.start_pos;
    if ext::mpCheckAllRemap(
        &mut hit_pos,
        &mut line_id,
        null_mut(),
        null_mut(),
        -1,
        -1,
        tmp.x,
        tmp.y,
        tmp.x,
        ext::Stage_GetBlastZoneBottomOffset(),
    ) != 0
        || ext::mpCheckAllRemap(
            &mut hit_pos,
            &mut line_id,
            null_mut(),
            null_mut(),
            -1,
            -1,
            tmp.x,
            ext::Stage_GetBlastZoneTopOffset(),
            tmp.x,
            tmp.y,
        ) != 0
    {
        spawn.prev_pos = hit_pos;
        spawn.pos = (*ip).xDD4_itemVar.houou.start_pos;
        spawn.facing_dir = (*ip).facing_dir;
        spawn.x3C_damage = 0;
        spawn.vel.x = 0.0;
        spawn.vel.y = 0.0;
        spawn.vel.z = 0.0;
        spawn.kind = It_Kind_Houou_SacredFire;
        spawn.x0_parent_gobj = (*ip).owner;
        spawn.x4_parent_gobj2 = gobj;
        spawn.x44_flag.anon_0.set_b0(1);
        spawn.x40 = 0;
        let new_gobj = ext::Item_80268B18(&mut spawn);
        (*ip_of(new_gobj)).xDD4_itemVar.houou.timer = line_id;
    }
}

/// `void it_2725_Logic42_Spawned(Item_GObj*)` (2D2E80)
pub unsafe extern "C" fn it_2725_Logic42_Spawned(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_2725_Logic42_Spawned");
    let item = ip_of(item_gobj);
    let attr = attr_of(item);
    (*item).xD44_lifeTimer = (*attr).timer;
    ext::it_80274740(item_gobj);
    it_802D2EF0(item_gobj);
    ext::it_8026B3A8(item_gobj);
}

/// `void it_802D2ED0(Item_GObj*, Item_GObj*)` (2D2ED0)
pub unsafe extern "C" fn it_802D2ED0(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D2ED0");
    ext::it_8026B894(gobj, ref_gobj);
}

/// `void it_802D2EF0(Item_GObj*)` (2D2EF0)
pub unsafe extern "C" fn it_802D2EF0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D2EF0");
    let ip = ip_of(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(ip);
}

/// `bool it_802D2F3C(Item_GObj*)` (2D2F3C): `Item_TickLifetime`.
pub unsafe extern "C" fn it_802D2F3C(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D2F3C");
    let ip = ip_of(gobj);
    if (*ip).xD44_lifeTimer <= 0.0 {
        return 1;
    }
    (*ip).xD44_lifeTimer -= 1.0;
    0
}

/// `void it_802D2F70(Item_GObj*)` (2D2F70)
pub unsafe extern "C" fn it_802D2F70(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D2F70");
    let mut sp10 = Vec { x: 0.0, y: 0.0, z: 0.0 };
    let ip = ip_of(gobj);

    if ext::mpLib_80054ED8((*ip).xDD4_itemVar.houou.timer) != 0 {
        ext::mpGetSpeed((*ip).xDD4_itemVar.houou.timer, &mut (*ip).pos, &mut sp10);
        (*ip).pos.x += sp10.x;
        (*ip).pos.y += sp10.y;
        (*ip).pos.z += sp10.z;
    }
}

/// `bool it_802D2FE8(Item_GObj*)` (2D2FE8)
pub unsafe extern "C" fn it_802D2FE8(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D2FE8");
    ext::it_8026DA08(gobj)
}
