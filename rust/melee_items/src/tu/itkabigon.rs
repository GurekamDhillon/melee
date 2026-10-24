//! Native port of src/melee/it/kinds/itkabigon.c (Snorlax Pokemon). Callees in other TUs go
//! through `ext`. Checked against the C (whole TU, shipping contraction) by tests/tu_itkabigon.rs.
//! Fused sites (`fmadds`): `it_802CA3F4` 2 (`2 * x18 * r - x18`).
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
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec);
    fn Camera_StopQuake(a0: u32);
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjClearFlagsAll(a0, a1) };
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn Stage_GetBlastZoneBottomOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneBottomOffset() };
    fn Stage_GetBlastZoneTopOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneTopOffset() };
    fn efLib_PauseAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::eflib::efLib_PauseAll(a0) };
    fn efLib_ResumeAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::eflib::efLib_ResumeAll(a0) };
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: &[*const c_void]) -> *mut c_void;
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
/// `HSD_JObjSetScale` (jobj.h)
#[inline]
unsafe fn jobj_set_scale(jobj: *mut HSD_JObj, scale: *const Vec3) {
    assert!(!jobj.is_null(), "jobj");
    assert!(!scale.is_null(), "scale");
    (*jobj).scale = *scale;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 && !jobj.is_null() && !jobj_mtx_is_dirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

/// `ItemStateTable it_803F7AD8[3]` (3F7AD8)
pub static it_803F7AD8: [ItemStateTable; 3] = [
    ItemStateTable { anim_id: 0, animated: Some(itKabigon_UnkMotion0_Anim), physics_updated: Some(itKabigon_UnkMotion0_Phys), collided: Some(itKabigon_UnkMotion0_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itKabigon_UnkMotion1_Anim), physics_updated: Some(itKabigon_UnkMotion1_Phys), collided: Some(itKabigon_UnkMotion1_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itKabigon_UnkMotion2_Anim), physics_updated: Some(itKabigon_UnkMotion2_Phys), collided: Some(itKabigon_UnkMotion2_Coll) },
];

/// `void it_802C9D40(Item_GObj*)` (2C9D40)
pub unsafe extern "C" fn it_802C9D40(gobj: *mut Item_GObj) {
    let ip = ip_of(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itKabigonAttributes;
    (*ip).xDD4_itemVar.kabigon.timer = (*attrs).x4;
    (*ip).xDD4_itemVar.kabigon.x64 = (*attrs).x8;
    (*ip).xDD4_itemVar.kabigon.x68 = (*attrs).xC;
    (*ip).xDD4_itemVar.kabigon.x6C = (*attrs).x10;
    (*ip).xDD4_itemVar.kabigon.xE44 = (*attrs).x14;
    (*ip).xDD4_itemVar.kabigon.xE48 = (*attrs).x18 as i32;
    (*ip).xDCC_flag.set_b3(1);
    (*ip).xDAC_itcmd_var0 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDB4_itcmd_var2 = 1;
    ext::it_80279CDC(gobj, (*attrs).x0);
    ext::Item_8026AE84(ip, 0x2723, 0x7F, 0x40);
    it_802CA324(gobj);
}

/// `void it_802C9DFC(Item_GObj*)` (2C9DFC)
pub unsafe extern "C" fn it_802C9DFC(_gobj: *mut Item_GObj) {}

/// `void itKabigon_Logic2_Destroyed(Item_GObj*)` (2C9E00)
pub unsafe extern "C" fn itKabigon_Logic2_Destroyed(_gobj: *mut Item_GObj) {
    ext::Camera_StopQuake(QuakeKind_Small);
}

/// `void it_802C9E24(Item_GObj*, Item_GObj*)` (2C9E24)
pub unsafe extern "C" fn it_802C9E24(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}

/// `void it_802C9E44(Item_GObj*)` (2C9E44)
pub unsafe extern "C" fn it_802C9E44(gobj: *mut Item_GObj) {
    let ip = ip_of(gobj);
    (*ip).xDCC_flag.set_b4567(0xE);
    ext::it_80273454(gobj);
    it_802C9E8C(gobj);
}

/// `void it_802C9E8C(Item_GObj*)` (2C9E8C)
pub unsafe extern "C" fn it_802C9E8C(gobj: *mut Item_GObj) {
    let ip = ip_of(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(ip);
    (*ip).on_accessory = Some(it_802CA014);
    ext::Camera_RequestQuake(QuakeKind_Medium, null_mut());
}

/// `bool itKabigon_UnkMotion0_Anim(Item_GObj*)` (2C9EF0)
pub unsafe extern "C" fn itKabigon_UnkMotion0_Anim(gobj: *mut Item_GObj) -> i32 {
    let ip = ip_of(gobj);

    if (*ip).xDB0_itcmd_var1 != 0 {
        (*ip).x40_vel.y = (*ip).xDD4_itemVar.kabigon.timer;
        ext::it_802762BC(ip);
        if (*ip).xDB4_itcmd_var2 != 0 {
            (*ip).xDB4_itcmd_var2 = 0;
        }
    }

    if (*ip).pos.y > ext::Stage_GetBlastZoneTopOffset() {
        it_802CA074(gobj);
    }

    0
}

/// `void itKabigon_UnkMotion0_Phys(Item_GObj*)` (2C9F74)
pub unsafe extern "C" fn itKabigon_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let ip = ip_of(gobj);

    if (*ip).xDB0_itcmd_var1 == 0 && (*ip).ground_or_air == 1 {
        let attrs = (*ip).xCC_item_attr;
        ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
    }
}

/// `bool itKabigon_UnkMotion0_Coll(Item_GObj*)` (2C9FBC)
pub unsafe extern "C" fn itKabigon_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    let ip = ip_of(gobj);

    if (*ip).xDB0_itcmd_var1 == 0 {
        if (*ip).ground_or_air == GA_Air {
            ext::it_8026E15C(gobj, Some(it_802C9DFC));
        } else {
            ext::it_8026D62C(gobj, Some(it_802C9DFC));
        }
    }
    0
}

/// `void it_802CA014(Item_GObj*)` (2CA014)
pub unsafe extern "C" fn it_802CA014(gobj: *mut Item_GObj) {
    let ip = (*gobj).user_data as *mut Item;
    if (*ip).xDB0_itcmd_var1 != 0 {
        (*ip).xDD4_itemVar.kabigon.xE44 = (*ip).xDD4_itemVar.kabigon.xE44.wrapping_sub(1);
        let timer = (*ip).xDD4_itemVar.kabigon.xE44;
        if timer == 0 {
            let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPokemonAttributes;
            it_802CA3F4(gobj);
            (*ip).xDD4_itemVar.kabigon.xE44 = (*attr).x14;
        }
    }
}

/// `void it_802CA074(Item_GObj*)` (2CA074)
pub unsafe extern "C" fn it_802CA074(gobj: *mut Item_GObj) {
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let ip = ip_of(gobj);

    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(ip);
    (*ip).x40_vel.y = 0.0;
    let s = (*ip).xDD4_itemVar.kabigon.x68;
    let scale = Vec { x: s, y: s, z: s };
    jobj_set_scale(jobj, &scale);
    ext::Item_8026AE84(ip, 0x2724, 0x7F, 0x40);
}

/// `static inline void itKabigon_UnkMotion1_Anim_inline(Item_GObj*, Item*)`
#[inline]
pub unsafe fn itKabigon_UnkMotion1_Anim_inline(gobj: *mut Item_GObj, ip: *mut Item) {
    if ext::it_80272C6C(gobj) == 0 {
        let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
        set_effect_hitlag_callbacks(ip);
        let s = (*ip).xDD4_itemVar.kabigon.x68;
        let scale = Vec { x: s, y: s, z: s };
        jobj_set_scale(jobj, &scale);
    }
}

/// `bool itKabigon_UnkMotion1_Anim(Item_GObj*)` (2CA198)
pub unsafe extern "C" fn itKabigon_UnkMotion1_Anim(gobj: *mut Item_GObj) -> i32 {
    let ip = (*gobj).user_data as *mut Item;
    if (*ip).xDAC_itcmd_var0 == 0 {
        if (*ip).xDD4_itemVar.kabigon.x6C <= 0 {
            let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
            (*ip).xDAC_itcmd_var0 = 1;
            ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
            (*ip).pos.y = ext::Stage_GetBlastZoneTopOffset();
            (*ip).x40_vel.y = (*ip).xDD4_itemVar.kabigon.x64;
            ext::Camera_RequestQuake(QuakeKind_Small, null_mut());
        } else {
            (*ip).xDD4_itemVar.kabigon.x6C = (*ip).xDD4_itemVar.kabigon.x6C.wrapping_sub(1);
        }
    } else if (*ip).pos.y < ext::Stage_GetBlastZoneBottomOffset() {
        return 1;
    }
    itKabigon_UnkMotion1_Anim_inline(gobj, ip);
    0
}

/// `void itKabigon_UnkMotion1_Phys(Item_GObj*)` (2CA318)
pub unsafe extern "C" fn itKabigon_UnkMotion1_Phys(_gobj: *mut Item_GObj) {}

/// `bool itKabigon_UnkMotion1_Coll(Item_GObj*)` (2CA31C)
pub unsafe extern "C" fn itKabigon_UnkMotion1_Coll(_gobj: *mut Item_GObj) -> i32 {
    0
}

/// `void it_802CA324(Item_GObj*)` (2CA324): `Item_EnterAirStateWithHitlag(gobj, 2)`.
pub unsafe extern "C" fn it_802CA324(gobj: *mut Item_GObj) {
    let item = ip_of(gobj);
    ext::it_802762BC(item);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(item);
}

/// `bool itKabigon_UnkMotion2_Anim(Item_GObj*)` (2CA388)
pub unsafe extern "C" fn itKabigon_UnkMotion2_Anim(gobj: *mut Item_GObj) -> i32 {
    ext::it_80279FF8(gobj);
    0
}

/// `void itKabigon_UnkMotion2_Phys(Item_GObj*)` (2CA3AC)
pub unsafe extern "C" fn itKabigon_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    ext::it_8027A09C(gobj);
}

/// `bool itKabigon_UnkMotion2_Coll(Item_GObj*)` (2CA3CC)
pub unsafe extern "C" fn itKabigon_UnkMotion2_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8027A118(gobj, Some(it_802C9E44))
}

/// `void it_802CA3F4(Item_GObj*)` (2CA3F4)
pub unsafe extern "C" fn it_802CA3F4(gobj: *mut Item_GObj) {
    let ip = ip_of(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itKabigonAttributes;
    let mut pos = (*ip).pos;

    let r = ext::HSD_Randf();
    pos.x += fmadds(2.0 * (*attr).x18, r, -(*attr).x18);
    let r = ext::HSD_Randf();
    pos.y += fmadds(2.0 * (*attr).x18, r, -(*attr).x18);

    ext::efSync_Spawn(0x405, gobj, &[&raw const pos as *const c_void]);
}
