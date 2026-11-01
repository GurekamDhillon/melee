//! Port of src/melee/it/kinds/itchicorita.c: Chikorita (Pokeball Pokemon, logic table 1) and its leaf
//! projectile (logic table 30). `chicorita.x60` counts the remaining leaf volleys, `x64` is its own
//! fall speed; `xDBC_itcmd_var4.flags.x0` is the event-script "throw a leaf" flag. Callees of unported
//! TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping
//! contraction) by tests/kinds_itchicorita.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

use super::kinds_inl::*;

// // BEGIN GENERATED EXT (rust/melee_items/gen/kinds/gen_tu.py; the callees of the C TU)
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
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB88(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB88(a0, a1 as _) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DFB0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DFB0(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80274740(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274740(a0) };
    fn it_80274C60(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80274c60(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80279C48(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279C48(a0) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_80279FF8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279FF8(a0) };
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027A09C(a0) as _ };
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::it_279C::it_8027A118(a0, a1) as _ };
    fn it_8027A160(a0: *mut HSD_JObj, a1: *mut Item) => unsafe { crate::tu::it_279C::it_8027A160(a0, a1) };
    fn it_8027A344(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_8027a344(&mut *((*a0).user_data as *mut Item)) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `ItemStateTable it_803F7A98[]`
pub static mut it_803F7A98: [ItemStateTable; 3] = [
    st_row(0, pred_b!(itChicorita_UnkMotion0_Anim), Some(itChicorita_UnkMotion0_Phys), pred_b!(itChicorita_UnkMotion0_Coll)),
    st_row(1, pred_b!(itChicorita_UnkMotion1_Anim), Some(itChicorita_UnkMotion1_Phys), pred_b!(itChicorita_UnkMotion1_Coll)),
    st_row(-1, pred_b!(itChicorita_UnkMotion2_Anim), Some(itChicorita_UnkMotion2_Phys), pred_b!(itChicorita_UnkMotion2_Coll)),
];

/// `ItemStateTable it_803F7AC8[]`
pub static mut it_803F7AC8: [ItemStateTable; 1] = [
    st_row(0, pred_b!(itChicoritaleaf_UnkMotion0_Anim), Some(itChicoritaleaf_UnkMotion0_Phys), pred_b!(itChicoritaleaf_UnkMotion0_Coll)),
];

/// 2C9588
pub unsafe extern "C" fn it_802C9588(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802C9588");
    let item = GET_ITEM(item_gobj);
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itChicoritaAttr;
    ext::it_80279C48(item_gobj);
    (*item).xDD4_itemVar.chicorita.x60 = -1;
    (*item).anon_dbc.xDBC_itcmd_var4.flags.set_x0(0);
    ext::it_80279CDC(item_gobj, (*attr).scale);
    (*item).xDD4_itemVar.chicorita.x64 = 0.0;
    it_802C9A74(item_gobj);
    ext::Item_8026AE84(item, 0x2711, 0x7F, 0x40);
}

/// 2C9618
pub unsafe extern "C" fn it_802C9618(_item_gobj: *mut HSD_GObj) { melee_core::ct!("it_802C9618");}

pub unsafe extern "C" fn itChicorita_Logic1_EvtUnk(item_gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("itChicorita_Logic1_EvtUnk");
    ext::it_8026B894(item_gobj, ref_gobj);
}

/// 2C963C
pub unsafe extern "C" fn it_802C963C(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802C963C");
    ext::it_80273454(item_gobj);
    it_802C9670(item_gobj);
}

/// 2C9670
pub unsafe extern "C" fn it_802C9670(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802C9670");
    let item = (*item_gobj).user_data as *mut Item;
    let article = (*item).xC4_article_data;
    let attr = (*article).x4_specialAttributes as *mut itChicoritaAttr;
    if (*item).xDD4_itemVar.chicorita.x60 == -1 {
        (*item).xDD4_itemVar.chicorita.x60 = (*attr).x4;
    }
    ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
    (*item).on_accessory = Some(it_802C989C);
}

pub unsafe extern "C" fn itChicorita_UnkMotion0_Anim(item_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itChicorita_UnkMotion0_Anim");
    let mut item_jobj = GET_JOBJ(item_gobj);
    let mut item = GET_ITEM(item_gobj);
    if item_jobj.is_null() {
        item_jobj = null_mut();
    } else {
        item_jobj = (*item_jobj).child;
    }
    ext::it_8027A160(item_jobj, item);
    if ext::it_80272C6C(item_gobj) == 0 {
        if (*item).xDD4_itemVar.chicorita.x60 <= 0 {
            (*item).xDD4_itemVar.chicorita.x60 = 0;
            it_802C98E4(item_gobj);
        } else {
            (*item).xDD4_itemVar.chicorita.x60 = (*item).xDD4_itemVar.chicorita.x60.wrapping_sub(1);
            item = (*item_gobj).user_data as *mut Item;
            let article = (*item).xC4_article_data;
            let attr = (*article).x4_specialAttributes as *mut itChicoritaAttr;
            if (*item).xDD4_itemVar.chicorita.x60 == -1 {
                (*item).xDD4_itemVar.chicorita.x60 = (*attr).x4;
            }
            ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
            (*item).on_accessory = Some(it_802C989C);
        }
    }
    false
}

/// `static inline itChicorita_Phys`
#[inline]
unsafe fn itChicorita_Phys(item_gobj: *mut HSD_GObj) {
    let item = (*item_gobj).user_data as *mut Item;
    ext::it_8027A344(item_gobj);
    if (*item).ground_or_air == GA_Air {
        let fall_speed_dir: i32 = if (*(*item).xCC_item_attr).x10_fall_speed < 0.0 { -1 } else { 1 };
        let mut chicorita_fall_speed = (*item).xDD4_itemVar.chicorita.x64;
        let item_vel_y_dir: i32 = if chicorita_fall_speed < 0.0 { -1 } else { 1 };
        if item_vel_y_dir != fall_speed_dir {
            if chicorita_fall_speed < 0.0 {
                chicorita_fall_speed = -chicorita_fall_speed;
            }
            if chicorita_fall_speed < (*(*item).xCC_item_attr).x14_fall_speed_max {
                (*item).xDD4_itemVar.chicorita.x64 -= (*(*item).xCC_item_attr).x10_fall_speed;
            }
        } else {
            (*item).xDD4_itemVar.chicorita.x64 -= (*(*item).xCC_item_attr).x10_fall_speed;
        }
        (*item).x40_vel.y = (*item).xDD4_itemVar.chicorita.x64;
        return;
    }
    (*item).xDD4_itemVar.chicorita.x64 = 0.0;
}

pub unsafe extern "C" fn itChicorita_UnkMotion0_Phys(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("itChicorita_UnkMotion0_Phys");
    itChicorita_Phys(item_gobj);
}

pub unsafe extern "C" fn itChicorita_UnkMotion0_Coll(item_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itChicorita_UnkMotion0_Coll");
    if (*((*item_gobj).user_data as *mut Item)).ground_or_air == GA_Air {
        ext::it_8026E15C(item_gobj, Some(it_802C9618));
    } else {
        ext::it_8026D62C(item_gobj, Some(it_802C9618));
    }
    false
}

/// 2C989C: the accessory callback; throws a leaf when the event script raised the flag.
pub unsafe extern "C" fn it_802C989C(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802C989C");
    let item = (*item_gobj).user_data as *mut Item;
    if (*item).anon_dbc.xDBC_itcmd_var4.flags.x0() != 0 {
        it_802C9B20(item_gobj);
        (*item).anon_dbc.xDBC_itcmd_var4.flags.set_x0(0);
    }
}

/// 2C98E4
pub unsafe extern "C" fn it_802C98E4(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802C98E4");
    ext::Item_80268E5C(item_gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itChicorita_UnkMotion1_Anim(item_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itChicorita_UnkMotion1_Anim");
    let mut item_jobj = GET_JOBJ(item_gobj);
    let item = GET_ITEM(item_gobj);
    if item_jobj.is_null() {
        item_jobj = null_mut();
    } else {
        item_jobj = (*item_jobj).child;
    }
    ext::it_8027A160(item_jobj, item);
    if ext::it_80272C6C(item_gobj) == 0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn itChicorita_UnkMotion1_Phys(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("itChicorita_UnkMotion1_Phys");
    itChicorita_Phys(item_gobj);
}

pub unsafe extern "C" fn itChicorita_UnkMotion1_Coll(item_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itChicorita_UnkMotion1_Coll");
    if (*((*item_gobj).user_data as *mut Item)).ground_or_air == GA_Air {
        ext::it_8026E15C(item_gobj, Some(it_802C9618));
    } else {
        ext::it_8026D62C(item_gobj, Some(it_802C9618));
    }
    false
}

/// 2C9A74
pub unsafe extern "C" fn it_802C9A74(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802C9A74");
    ext::it_802762BC((*item_gobj).user_data as *mut Item);
    ext::Item_80268E5C(item_gobj, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itChicorita_UnkMotion2_Anim(item_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itChicorita_UnkMotion2_Anim");
    ext::it_80279FF8(item_gobj);
    false
}

pub unsafe extern "C" fn itChicorita_UnkMotion2_Phys(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("itChicorita_UnkMotion2_Phys");
    ext::it_8027A09C(item_gobj);
}

pub unsafe extern "C" fn itChicorita_UnkMotion2_Coll(item_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itChicorita_UnkMotion2_Coll");
    ext::it_8027A118(item_gobj, Some(it_802C963C)) != 0
}

/// 2C9B20: throws a leaf (1 fused site: `prev_pos.x += x8 * facing_dir`).
pub unsafe extern "C" fn it_802C9B20(chicorita_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802C9B20");
    let mut spawn: SpawnItem = core::mem::zeroed();

    let chicorita = GET_ITEM(chicorita_gobj);
    let attr = (*(*chicorita).xC4_article_data).x4_specialAttributes as *mut itChicoritaLeafAttr;
    spawn.prev_pos = (*chicorita).pos;
    spawn.prev_pos.x = fmadds((*attr).x8, (*chicorita).facing_dir, spawn.prev_pos.x);
    spawn.prev_pos.y += (*attr).xC;
    ext::it_8026BB88(chicorita_gobj, &mut spawn.pos);
    spawn.facing_dir = (*chicorita).facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.x = (*attr).x10 * (*chicorita).facing_dir;
    spawn.vel.y = 0.0;
    spawn.vel.z = 0.0;
    spawn.x40 = 0;
    spawn.kind = It_Kind_Chicorita_Leaf;
    spawn.x0_parent_gobj = (*chicorita).owner;
    spawn.x4_parent_gobj2 = chicorita_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    if !ext::Item_80268B18(&mut spawn).is_null() {
        ext::Item_8026AE84(chicorita, 0x2710, 0x7F, 0x40);
    }
}

pub unsafe extern "C" fn itChicoritaLeaf_Logic30_Spawned(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itChicoritaLeaf_Logic30_Spawned");
    let item = GET_ITEM(item_gobj);
    (*item).xD44_lifeTimer = (*((*(*item).xC4_article_data).x4_specialAttributes as *mut itChicoritaLeafAttr)).timer;
    ext::it_80274740(item_gobj);
    it_802C9CC0(item_gobj);
    ext::it_8026B3A8(item_gobj);
    ext::it_80274C60(item_gobj);
}

pub unsafe extern "C" fn itChicoritaLeaf_Logic30_HitShield(_item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itChicoritaLeaf_Logic30_HitShield");
    false
}

pub unsafe extern "C" fn itChicoritaLeaf_Logic30_EvtUnk(item_gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("itChicoritaLeaf_Logic30_EvtUnk");
    ext::it_8026B894(item_gobj, ref_gobj);
}

pub unsafe extern "C" fn itChicoritaLeaf_Logic30_Reflected(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itChicoritaLeaf_Logic30_Reflected");
    ext::it_80273030(item_gobj) != 0
}

/// 2C9CC0
pub unsafe extern "C" fn it_802C9CC0(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802C9CC0");
    ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itChicoritaleaf_UnkMotion0_Anim(item_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itChicoritaleaf_UnkMotion0_Anim");
    let item = (*item_gobj).user_data as *mut Item;
    if (*item).xD44_lifeTimer <= 0.0 {
        return true;
    }
    (*item).xD44_lifeTimer -= 1.0;
    false
}

pub unsafe extern "C" fn itChicoritaleaf_UnkMotion0_Phys(_item_gobj: *mut HSD_GObj) { melee_core::ct!("itChicoritaleaf_UnkMotion0_Phys");}

pub unsafe extern "C" fn itChicoritaleaf_UnkMotion0_Coll(item_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itChicoritaleaf_UnkMotion0_Coll");
    ext::it_8026DFB0(item_gobj) != 0
}
