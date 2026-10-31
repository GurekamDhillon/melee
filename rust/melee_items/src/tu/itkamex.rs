//! Native port of src/melee/it/kinds/itkamex.c (Blastoise Pokemon and its hydro pump). Callees in
//! other TUs go through `ext`. Checked against the C (whole TU, shipping contraction) by
//! tests/tu_itkamex.rs.
//! Fused sites (`fmadds`): `itKamex_UnkMotion1_Phys` 1 (`vel.x += x6C * facing`), `it_802CAB10` 3.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efLib_PauseAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_PauseAll(a0) };
    fn efLib_ResumeAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_ResumeAll(a0) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB88(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB88(a0, a1 as _) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DFB0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DFB0(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80274740(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274740(a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80279C48(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279C48(a0) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_80279FF8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279FF8(a0) };
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027A09C(a0) as _ };
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::it_279C::it_8027A118(a0, a1) as _ };
    fn it_8027ADEC(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj, a3: f32) => unsafe { crate::tu::it_279C::it_8027ADEC(a0, a1, a2, a3) };
    fn lbAudioAx_800263E8(a0: f32, a1: *mut HSD_GObj, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: i32, a8: i32, a9: i32, a10: i32) -> *mut HSD_GObj;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn efLib_PauseAll(a0: *mut HSD_GObj) { super::efLib_PauseAll(a0) }
        pub unsafe extern "C" fn efLib_ResumeAll(a0: *mut HSD_GObj) { super::efLib_ResumeAll(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut item: *mut Item = unsafe { core::mem::zeroed() };
    pub static mut result: i32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

#[inline]
unsafe fn ip_of(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}
#[inline]
unsafe fn set_effect_hitlag_callbacks(ip: *mut Item) {
    (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}
#[inline]
unsafe fn jobj_mtx_is_dirty(jobj: *mut HSD_JObj) -> bool {
    assert!(!jobj.is_null(), "jobj");
    (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
}
#[inline]
unsafe fn jobj_set_scale(jobj: *mut HSD_JObj, scale: *const Vec3) {
    assert!(!jobj.is_null(), "jobj");
    assert!(!scale.is_null(), "scale");
    (*jobj).scale = *scale;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 && !jobj.is_null() && !jobj_mtx_is_dirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}
#[inline]
unsafe fn jobj_get_scale(jobj: *mut HSD_JObj, scale: *mut Vec3) {
    assert!(!jobj.is_null(), "jobj");
    assert!(!scale.is_null(), "scale");
    *scale = (*jobj).scale;
}

/// `ItemStateTable it_803F7B08[4]` (3F7B08)
pub static it_803F7B08: [ItemStateTable; 4] = [
    ItemStateTable { anim_id: 0, animated: Some(it_802CA5D8), physics_updated: Some(it_802CA618), collided: Some(it_802CA654) },
    ItemStateTable { anim_id: 1, animated: Some(itKamex_UnkMotion1_Anim), physics_updated: Some(itKamex_UnkMotion1_Phys), collided: Some(itKamex_UnkMotion1_Coll) },
    ItemStateTable { anim_id: 2, animated: Some(itKamex_UnkMotion2_Anim), physics_updated: Some(itKamex_UnkMotion2_Phys), collided: Some(itKamex_UnkMotion2_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itKamex_UnkMotion3_Anim), physics_updated: Some(itKamex_UnkMotion3_Phys), collided: Some(itKamex_UnkMotion3_Coll) },
];

/// `ItemStateTable it_803F7B48[1]` (3F7B48)
pub static it_803F7B48: [ItemStateTable; 1] =
    [ItemStateTable { anim_id: 0, animated: Some(it_802CAE60), physics_updated: Some(it_802CAE94), collided: Some(it_802CAFB4) }];

/// `void it_802CA49C(Item_GObj*)` (2CA49C)
pub unsafe extern "C" fn it_802CA49C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CA49C");
    let ip = ip_of(gobj);
    let sa = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPokemonAttributes;

    ext::it_80279C48(gobj);
    (*ip).xDD4_itemVar.pokemon.timer = -1;
    (*ip).anon_dbc.xDBC_itcmd_var4.flags.set_x0(0);
    (*ip).xDD4_itemVar.pokemon.x64 = 0;
    (*ip).xDD4_itemVar.pokemon.x68 = 0.0;
    (*ip).xDD4_itemVar.pokemon.x6C = 0.0;
    ext::it_80279CDC(gobj, (*sa).x0);
    it_802CAA40(gobj);
    ext::Item_8026AE84(ip, 0x271E, 127, 64);
}

/// `void it_802CA534(Item_GObj*)` (2CA534)
pub unsafe extern "C" fn it_802CA534(_gobj: *mut Item_GObj) { melee_core::ct!("it_802CA534");}

/// `void it_802CA538(Item_GObj*, Item_GObj*)` (2CA538)
pub unsafe extern "C" fn it_802CA538(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CA538");
    ext::it_8026B894(gobj, ref_gobj);
}

/// `void it_802CA558(Item_GObj*)` (2CA558)
pub unsafe extern "C" fn it_802CA558(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CA558");
    ext::it_80273454(gobj);
    it_802CA58C(gobj);
}

/// `void it_802CA58C(Item_GObj*)` (2CA58C)
pub unsafe extern "C" fn it_802CA58C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CA58C");
    let ip = ip_of(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(ip);
}

/// `bool it_802CA5D8(Item_GObj*)` (2CA5D8)
pub unsafe extern "C" fn it_802CA5D8(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802CA5D8");
    if ext::it_80272C6C(gobj) == 0 {
        it_802CA6A0(gobj);
    }
    0
}

/// `void it_802CA618(Item_GObj*)` (2CA618)
pub unsafe extern "C" fn it_802CA618(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CA618");
    let ip = ip_of(gobj);
    if (*ip).ground_or_air == GA_Air {
        ext::it_80272860(gobj, (*(*ip).xCC_item_attr).x10_fall_speed, (*(*ip).xCC_item_attr).x14_fall_speed_max);
    }
}

/// `bool it_802CA654(Item_GObj*)` (2CA654)
pub unsafe extern "C" fn it_802CA654(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802CA654");
    if (*ip_of(gobj)).ground_or_air == GA_Air {
        ext::it_8026E15C(gobj, Some(it_802CA534));
    } else {
        ext::it_8026D62C(gobj, Some(it_802CA534));
    }
    0
}

/// `void it_802CA6A0(Item_GObj*)` (2CA6A0)
pub unsafe extern "C" fn it_802CA6A0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CA6A0");
    let ip = ip_of(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itKamexAttributes;

    if (*ip).xDD4_itemVar.pokemon.timer == -1 {
        (*ip).xDD4_itemVar.pokemon.timer = (*attrs).timer;
        (*ip).xDD4_itemVar.pokemon.x68 = (*attrs).x18;
        (*ip).xDD4_itemVar.pokemon.x6C = (*attrs).x1C;
    }
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(ip);
    (*ip).on_accessory = Some(it_802CA8DC);
}

/// `bool itKamex_UnkMotion1_Anim(Item_GObj*)` (2CA724)
pub unsafe extern "C" fn itKamex_UnkMotion1_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKamex_UnkMotion1_Anim");
    let ip = (*gobj).user_data as *mut Item;

    if ext::it_80272C6C(gobj) == 0 {
        if (*ip).xDD4_itemVar.pokemon.timer <= 0 {
            (*ip).xDD4_itemVar.pokemon.timer = 0;
            (*ip).x40_vel.x = 0.0;
            it_802CA938(gobj);
        } else {
            (*ip).xDD4_itemVar.pokemon.timer -= 1;
            it_802CA6A0(gobj);
        }
    }
    0
}

/// `void itKamex_UnkMotion1_Phys(Item_GObj*)` (2CA7FC)
pub unsafe extern "C" fn itKamex_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itKamex_UnkMotion1_Phys");
    let ip = (*gobj).user_data as *mut Item;

    if (*ip).x40_vel.x != 0.0 {
        if ((*ip).x40_vel.x > 0.0 && (*ip).facing_dir <= 0.0) || ((*ip).x40_vel.x <= 0.0 && (*ip).facing_dir > 0.0) {
            (*ip).x40_vel.x = fmadds((*ip).xDD4_itemVar.pokemon.x6C, (*ip).facing_dir, (*ip).x40_vel.x);
        }
    }
    if (*ip).ground_or_air == GA_Air {
        ext::it_80272860(gobj, (*(*ip).xCC_item_attr).x10_fall_speed, (*(*ip).xCC_item_attr).x14_fall_speed_max);
    }
}

/// `bool itKamex_UnkMotion1_Coll(Item_GObj*)` (2CA890)
pub unsafe extern "C" fn itKamex_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKamex_UnkMotion1_Coll");
    if (*ip_of(gobj)).ground_or_air == GA_Air {
        ext::it_8026E15C(gobj, Some(it_802CA534));
    } else {
        ext::it_8026D62C(gobj, Some(it_802CA534));
    }
    0
}

/// `void it_802CA8DC(Item_GObj*)` (2CA8DC)
pub unsafe extern "C" fn it_802CA8DC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CA8DC");
    let ip = (*gobj).user_data as *mut Item;
    if (*ip).anon_dbc.xDBC_itcmd_var4.flags.x0() != 0 {
        it_802CAB10(gobj);
        (*ip).x40_vel.x = (*ip).xDD4_itemVar.pokemon.x68 * -(*ip).facing_dir;
        (*ip).anon_dbc.xDBC_itcmd_var4.flags.set_x0(0);
    }
}

/// `void it_802CA938(Item_GObj*)` (2CA938)
pub unsafe extern "C" fn it_802CA938(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CA938");
    let ip = ip_of(gobj);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(ip);
}

/// `bool itKamex_UnkMotion2_Anim(Item_GObj*)` (2CA984)
pub unsafe extern "C" fn itKamex_UnkMotion2_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKamex_UnkMotion2_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        return 1;
    }
    0
}

/// `void itKamex_UnkMotion2_Phys(Item_GObj*)` (2CA9B8)
pub unsafe extern "C" fn itKamex_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itKamex_UnkMotion2_Phys");
    let ip = ip_of(gobj);
    if (*ip).ground_or_air == GA_Air {
        ext::it_80272860(gobj, (*(*ip).xCC_item_attr).x10_fall_speed, (*(*ip).xCC_item_attr).x14_fall_speed_max);
    }
}

/// `bool itKamex_UnkMotion2_Coll(Item_GObj*)` (2CA9F4)
pub unsafe extern "C" fn itKamex_UnkMotion2_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKamex_UnkMotion2_Coll");
    if (*ip_of(gobj)).ground_or_air == GA_Air {
        ext::it_8026E15C(gobj, Some(it_802CA534));
    } else {
        ext::it_8026D62C(gobj, Some(it_802CA534));
    }
    0
}

/// `void it_802CAA40(Item_GObj*)` (2CAA40): `Item_EnterAirStateWithHitlag(gobj, 3)`.
pub unsafe extern "C" fn it_802CAA40(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CAA40");
    let item = ip_of(gobj);
    ext::it_802762BC(item);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(item);
}

/// `bool itKamex_UnkMotion3_Anim(Item_GObj*)` (2CAAA4)
pub unsafe extern "C" fn itKamex_UnkMotion3_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKamex_UnkMotion3_Anim");
    ext::it_80279FF8(gobj);
    0
}

/// `void itKamex_UnkMotion3_Phys(Item_GObj*)` (2CAAC8)
pub unsafe extern "C" fn itKamex_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itKamex_UnkMotion3_Phys");
    ext::it_8027A09C(gobj);
}

/// `bool itKamex_UnkMotion3_Coll(Item_GObj*)` (2CAAE8)
pub unsafe extern "C" fn itKamex_UnkMotion3_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKamex_UnkMotion3_Coll");
    ext::it_8027A118(gobj, Some(it_802CA558))
}

/// `void it_802CAB10(Item_GObj*)` (2CAB10): fire a hydro pump.
pub unsafe extern "C" fn it_802CAB10(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CAB10");
    let mut spawn: SpawnItem = core::mem::zeroed();
    let ip = ip_of(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itKamexAttributes;

    spawn.prev_pos = (*ip).pos;
    spawn.prev_pos.x = fmadds((*attrs).x8, (*ip).facing_dir, spawn.prev_pos.x);
    spawn.prev_pos.y += (*attrs).xC;
    if (*ip).xDD4_itemVar.pokemon.x64 != 0 {
        spawn.prev_pos.z = fmadds(-1.0, (*attrs).x10 * (*ip).facing_dir, spawn.prev_pos.z);
    } else {
        spawn.prev_pos.z = fmadds((*attrs).x10, (*ip).facing_dir, spawn.prev_pos.z);
    }
    ext::it_8026BB88(gobj, &mut spawn.pos);
    spawn.facing_dir = (*ip).facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.x = (*attrs).x14 * (*ip).facing_dir;
    spawn.vel.y = 0.0;
    spawn.vel.z = 0.0;
    spawn.kind = It_Kind_Kamex_HydroPump;
    (*ip).xDD4_itemVar.pokemon.x64 ^= 1;
    spawn.x0_parent_gobj = (*ip).owner;
    spawn.x4_parent_gobj2 = gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;
    if !ext::Item_80268B18(&mut spawn).is_null() {
        if (*ip).facing_dir == 1.0 {
            ext::lbAudioAx_800263E8(3.0, gobj, 1, 0x271D, 0x7F, 0x7F, 0, 0x7F, 0x3C, 0, -1);
            return;
        }
        ext::lbAudioAx_800263E8(3.0, gobj, 1, 0x271C, 0x7F, 0x7F, 0, 0x7F, 0x3C, 0, -1);
    }
}

/// `void it_2725_Logic31_Spawned(Item_GObj*)` (2CACD0)
pub unsafe extern "C" fn it_2725_Logic31_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("it_2725_Logic31_Spawned");
    let ip = (*gobj).user_data as *mut Item;
    let sa = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itKamexAttributes;

    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    (*ip).xD44_lifeTimer = (*sa).x0;
    ext::it_80274740(gobj);
    it_802CADF0(gobj);
    ext::it_8026B3A8(gobj);
    let scale = Vec { x: 0.2, y: 0.2, z: 0.2 };
    jobj_set_scale(jobj, &scale);
}

/// `bool itKamex_Logic31_HitShield(Item_GObj*)` (2CADC0)
pub unsafe extern "C" fn itKamex_Logic31_HitShield(_arg0: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKamex_Logic31_HitShield");
    1
}

/// `bool itKamex_Logic31_DmgDealt(Item_GObj*)` (2CADC8)
pub unsafe extern "C" fn itKamex_Logic31_DmgDealt(_arg0: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKamex_Logic31_DmgDealt");
    0
}

/// `void itKamex_Logic31_EvtUnk(Item_GObj*, Item_GObj*)` (2CADD0)
pub unsafe extern "C" fn itKamex_Logic31_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itKamex_Logic31_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}

/// `void it_802CADF0(Item_GObj*)` (2CADF0)
pub unsafe extern "C" fn it_802CADF0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CADF0");
    let ip = ip_of(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    ext::it_8027ADEC(0x46A, gobj, jobj, 1.2);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(ip);
}

/// `bool it_802CAE60(Item_GObj*)` (2CAE60)
pub unsafe extern "C" fn it_802CAE60(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802CAE60");
    let ip = ip_of(gobj);
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        return 1;
    }
    0
}

/// `void it_802CAE94(Item_GObj*)` (2CAE94)
pub unsafe extern "C" fn it_802CAE94(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CAE94");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    if ((*jobj).scale.x as f64) < 1.0 {
        let mut scale = Vec { x: 0.0, y: 0.0, z: 0.0 };
        jobj_get_scale(jobj, &mut scale);
        scale.x = (scale.x as f64 + 0.06) as f32;
        scale.y = (scale.y as f64 + 0.06) as f32;
        scale.z = (scale.z as f64 + 0.06) as f32;
        jobj_set_scale(jobj, &scale);
    }
}

/// `bool it_802CAFB4(Item_GObj*)` (2CAFB4)
pub unsafe extern "C" fn it_802CAFB4(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802CAFB4");
    ext::it_8026DFB0(gobj)
}
