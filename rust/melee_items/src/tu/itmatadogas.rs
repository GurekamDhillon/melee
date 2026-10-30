//! Port of src/melee/it/kinds/itmatadogas.c: Matadogas (Pokeball Pokemon, logic table 32) and its gas
//! clouds (logic tables 32/33; `it_803F7B58`, `it_803F7BB0`). `matadogas.x60` counts frames to the next
//! cloud, `x64` toggles the voice. Callees of unported TUs go through `ext` (unset: panics "TODO").
//! Checked against the C (whole TU, shipping contraction) by tests/kinds_itmatadogas.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_runtime::gekko::{cosf, sinf};
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
    fn HSD_JObjSetMtxDirty(a0: *mut HSD_JObj) => unsafe { crate::tu::item::HSD_JObjSetMtxDirty(a0) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efLib_PauseAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::eflib::efLib_PauseAll(a0) };
    fn efLib_ResumeAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::eflib::efLib_ResumeAll(a0) };
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj, a3: *mut f32) -> *mut c_void => unsafe { melee_world::tu_ext::efSync_Spawn(a0, a1, a2, a3) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB88(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB88(a0, a1 as _) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80274740(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274740(a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_80279FF8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279FF8(a0) };
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027A09C(a0) as _ };
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::it_279C::it_8027A118(a0, a1) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirty(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirty(a0) }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn efLib_PauseAll(a0: *mut HSD_GObj) { super::efLib_PauseAll(a0) }
        pub unsafe extern "C" fn efLib_ResumeAll(a0: *mut HSD_GObj) { super::efLib_ResumeAll(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

kinds_jobj_inl!();

/// `ItemStateTable it_803F7B58[]`
pub static mut it_803F7B58: [ItemStateTable; 3] = [
    st_row(0, pred_b(it_802CB118), Some(it_802CB14C), pred_b(it_802CB150)),
    st_row(1, pred_b(itMatadogas_UnkMotion1_Anim), Some(itMatadogas_UnkMotion1_Phys), pred_b(itMatadogas_UnkMotion1_Coll)),
    st_row(-1, pred_b(itMatadogas_UnkMotion2_Anim), Some(itMatadogas_UnkMotion2_Phys), pred_b(itMatadogas_UnkMotion2_Coll)),
];

/// `ItemStateTable it_803F7BB0[]`
pub static mut it_803F7BB0: [ItemStateTable; 1] = [st_row(0, pred_b(it_802CB810), Some(it_802CB844), pred_b(it_802CB8A4))];

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

/// `Item_TickLifetime` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_TickLifetime(ip: *mut Item) -> bool {
    if (*ip).xD44_lifeTimer <= 0.0 {
        return true;
    }
    (*ip).xD44_lifeTimer -= 1.0;
    false
}

/// 2CAFD4
pub unsafe extern "C" fn it_802CAFD4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CAFD4");
    let item = (*gobj).user_data as *mut Item;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itMatadogasAttributes;
    (*item).xDAC_itcmd_var0 = 0;
    (*item).xDD4_itemVar.matadogas.x60 = (*attrs).x4 as i32;
    (*item).xDD4_itemVar.matadogas.x64 = 0;
    ext::it_80279CDC(gobj, (*attrs).x0);
    it_802CB350(gobj);
    HSD_JObjSetRotationY(jobj, 0.0);
    ext::Item_8026AE84(item, 0x2715, 0x7f, 0x40);
}

/// 2CB0F4
pub unsafe extern "C" fn it_802CB0F4(_gobj: *mut Item_GObj) { melee_core::ct!("it_802CB0F4");}

/// 2CB0F8
pub unsafe extern "C" fn it_802CB0F8(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CB0F8");
    ext::it_8026B894(gobj, ref_gobj);
}

/// 2CB118
pub unsafe extern "C" fn it_802CB118(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802CB118");
    ext::it_80272C6C(gobj) == 0
}

/// 2CB14C
pub unsafe extern "C" fn it_802CB14C(_gobj: *mut Item_GObj) { melee_core::ct!("it_802CB14C");}

/// 2CB150
pub unsafe extern "C" fn it_802CB150(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802CB150");
    ext::it_8026E15C(gobj, Some(it_802CB0F4));
    false
}

pub unsafe extern "C" fn itMatadogas_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMatadogas_UnkMotion1_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        let ip = (*gobj).user_data as *mut Item;
        ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
        Item_SetEffectHitlagCallbacks(ip);
        HSD_JObjSetRotationY(jobj, 0.0);
    }
    false
}

pub unsafe extern "C" fn itMatadogas_UnkMotion1_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itMatadogas_UnkMotion1_Phys");}

pub unsafe extern "C" fn itMatadogas_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMatadogas_UnkMotion1_Coll");
    ext::it_8026E15C(gobj, Some(it_802CB0F4));
    false
}

/// 2CB2B0
pub unsafe extern "C" fn it_802CB2B0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CB2B0");
    let ip = GET_ITEM(gobj);
    if (*ip).xDAC_itcmd_var0 != 0 {
        let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMatadogasAttributes;
        (*ip).xDD4_itemVar.matadogas.x60 = (*ip).xDD4_itemVar.matadogas.x60.wrapping_sub(1);
        if (*ip).xDD4_itemVar.matadogas.x60 == 0 {
            (*ip).xDD4_itemVar.matadogas.x60 = (*attrs).x4 as i32;
            if ext::HSD_Randi(2) == 0 {
                it_802CB4F0(gobj, It_Kind_Matadogas_Gas1, (*attrs).x8);
            } else {
                it_802CB4F0(gobj, It_Kind_Matadogas_Gas2, (*attrs).xC);
            }
        }
    }
}

/// 2CB350
pub unsafe extern "C" fn it_802CB350(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CB350");
    Item_EnterAirStateWithHitlag(gobj, 2);
}

pub unsafe extern "C" fn itMatadogas_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMatadogas_UnkMotion2_Anim");
    ext::it_80279FF8(gobj);
    false
}

pub unsafe extern "C" fn itMatadogas_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itMatadogas_UnkMotion2_Phys");
    if ext::it_8027A09C(gobj) != 0 {
        let item = (*gobj).user_data as *mut Item;
        ext::it_80273454(gobj);
        let item2 = (*gobj).user_data as *mut Item;
        let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
        Item_SetEffectHitlagCallbacks(item2);
        hsd_assert(!jobj.is_null(), "jobj");
        hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
        (*jobj).rotate.y = 0.0;
        if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
            ext::HSD_JObjSetMtxDirty(jobj);
        }
        (*item2).on_accessory = Some(it_802CB2B0);
        (*item).xDD1_flag.anon_0.set_b1(1);
    }
}

pub unsafe extern "C" fn itMatadogas_UnkMotion2_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMatadogas_UnkMotion2_Coll");
    ext::it_8027A118(gobj, Some(it_802CB0F4)) != 0
}

/// 2CB4F0
pub unsafe extern "C" fn it_802CB4F0(gobj: *mut Item_GObj, kind: i32, radius: f32) {
    melee_core::ct!("it_802CB4F0");
    let ip = GET_ITEM(gobj);
    let mut spawn: SpawnItem = core::mem::zeroed();

    let rand: f32 = ext::HSD_Randi(360) as f32;
    spawn.prev_pos = (*ip).pos;
    ext::it_8026BB88(gobj, &mut spawn.pos);
    spawn.facing_dir = (*ip).facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.x = radius * cosf(rand * 0.01745329252f32);
    spawn.vel.y = radius * sinf(rand * 0.01745329252f32);
    spawn.vel.z = 0.0;
    spawn.kind = kind;
    spawn.x0_parent_gobj = (*ip).owner;
    spawn.x4_parent_gobj2 = gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;

    if !ext::Item_80268B18(&mut spawn).is_null() {
        (*ip).xDD4_itemVar.matadogas.x64 ^= 1;
        if (*ip).xDD4_itemVar.matadogas.x64 != 0 {
            match ext::HSD_Randi(3) {
                0 => ext::Item_8026AE84(ip, 0x2712, 0x7F, 0x40),
                1 => ext::Item_8026AE84(ip, 0x2713, 0x7F, 0x40),
                2 => ext::Item_8026AE84(ip, 0x2714, 0x7F, 0x40),
                _ => {}
            }
        }
    }
}

pub unsafe extern "C" fn it_2725_Logic32_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("it_2725_Logic32_Spawned");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut scale: f32 = 1.0;
    ext::efSync_Spawn(0x45B, gobj, jobj, &mut scale);
    ext::efSync_Spawn(0x45D, gobj, jobj, &mut scale);
    it_802CB798(gobj);
}

pub unsafe extern "C" fn it_2725_Logic33_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("it_2725_Logic33_Spawned");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut scale: f32 = 1.0;
    ext::efSync_Spawn(0x45C, gobj, jobj, &mut scale);
    ext::efSync_Spawn(0x45E, gobj, jobj, &mut scale);
    it_802CB798(gobj);
}

pub unsafe extern "C" fn itMatadogas_Logic33_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itMatadogas_Logic33_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}

/// 2CB798
pub unsafe extern "C" fn it_802CB798(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CB798");
    let ip = (*gobj).user_data as *mut Item;
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMatadogasAttributes;
    (*ip).xD44_lifeTimer = (*attrs).x0;
    ext::it_80274740(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    Item_SetEffectHitlagCallbacks(ip);
    ext::it_8026B3A8(gobj);
}

/// 2CB810
pub unsafe extern "C" fn it_802CB810(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802CB810");
    let ip = GET_ITEM(gobj);
    Item_TickLifetime(ip)
}

/// 2CB844
pub unsafe extern "C" fn it_802CB844(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802CB844");
    let ip = (*gobj).user_data as *mut Item;
    let mut scale: f32 = 0.0;
    let kind = (*ip).kind;
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMatadogasAttributes;

    match kind {
        It_Kind_Matadogas_Gas1 => scale = (*attrs).x4,
        It_Kind_Matadogas_Gas2 => scale = (*attrs).x8,
        _ => {}
    }

    (*ip).x40_vel.x *= scale;
    (*ip).x40_vel.y *= scale;
    (*ip).x40_vel.z *= scale;
}

/// 2CB8A4
pub unsafe extern "C" fn it_802CB8A4(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802CB8A4");
    false
}
