//! Port of src/melee/it/kinds/ityoshistar.c: the star spawned by Yoshi's egg pop (logic table 90). Callees of
//! unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction)
//! by tests/kinds_ityoshistar.rs.
//!
//! Fused site (checked in the IR): `x40_vel.x += star->accel * facing_dir` is one `fmadds`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
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
use melee_core::fp::fmadds;
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026E0F4(a0: *mut HSD_GObj) => unsafe { crate::tu::itgroundcoll::it_8026E0F4(a0) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272BA4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272BA4(a0) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

/// `ItemStateTable it_803F7158[]`
pub static mut it_803F7158: [ItemStateTable; 1] = [st_row(
    0,
    pred_b!(itYoshistar_UnkMotion0_Anim),
    Some(itYoshistar_UnkMotion0_Phys),
    pred_b!(itYoshistar_UnkMotion0_Coll),
)];

/// The file-local `StarAttrs` (the article's special attributes).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct StarAttrs {
    pub speed: f32,
    pub accel: f32,
}

/// `itReflectItemAndUpdateRotation` (it/kinds/inlines.h, static inline).
#[inline]
unsafe fn itReflectItemAndUpdateRotation(gobj: *mut Item_GObj) -> bool {
    const PI_2: f32 = (core::f64::consts::PI / 2.0) as f32;
    let ip = HSD_GObjGetUserData(gobj) as *mut Item;
    let jobj = GET_JOBJ(gobj);

    ext::it_80273030(gobj);
    HSD_JObjSetRotationY(jobj, PI_2 * (*ip).facing_dir);
    false
}

pub unsafe extern "C" fn it_802B2FC8(parent: *mut HSD_GObj, pos: *mut Vec3, dir: f32) -> *mut Item_GObj {
    melee_core::ct!("it_802B2FC8");
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = It_Kind_Yoshi_Star;
    spawn.prev_pos = *pos;
    ext::it_8026BB68(parent, &mut spawn.pos);
    // Item_InitSpawnCommonFields(&spawn, parent, dir, true)
    spawn.facing_dir = dir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = parent;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;
    let item_gobj = ext::Item_80268B18(&mut spawn);
    if !item_gobj.is_null() {
        it_802B322C(item_gobj, parent);
        ext::db_80225DD8(item_gobj, parent);
        return item_gobj;
    }
    null_mut()
}

pub unsafe extern "C" fn it_802B309C(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802B309C");
    ext::it_80272BA4(item_gobj);
    true
}

pub unsafe extern "C" fn it_802B30C0(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802B30C0");
    ext::it_80272BA4(item_gobj);
    true
}

pub unsafe extern "C" fn it_802B30E4(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802B30E4");
    ext::it_80272BA4(item_gobj);
    true
}

pub unsafe extern "C" fn it_802B3108(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802B3108");
    ext::it_80272BA4(item_gobj);
    true
}

pub unsafe extern "C" fn it_802B312C(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802B312C");
    ext::itColl_BounceOffShield(item_gobj) != 0
}

pub unsafe extern "C" fn it_802B314C(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802B314C");
    itReflectItemAndUpdateRotation(item_gobj)
}

pub unsafe extern "C" fn it_802B322C(item_gobj: *mut Item_GObj, owner: *mut HSD_GObj) {
    melee_core::ct!("it_802B322C");
    let item = GET_ITEM(item_gobj);
    let star = (*(*item).xC4_article_data).x4_specialAttributes as *mut StarAttrs;
    (*item).owner = owner;
    (*item).x40_vel.x = (*star).speed * (*item).facing_dir;
    (*item).x40_vel.y = (*(*item).xCC_item_attr).x18;
    ext::it_8026B3A8(item_gobj);
    ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itYoshistar_UnkMotion0_Anim(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itYoshistar_UnkMotion0_Anim");
    if ext::it_80272C6C(item_gobj) == 0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn itYoshistar_UnkMotion0_Phys(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itYoshistar_UnkMotion0_Phys");
    let item = GET_ITEM(item_gobj);
    let attr = (*item).xCC_item_attr;
    let star = (*(*item).xC4_article_data).x4_specialAttributes as *mut StarAttrs;
    ext::it_80272860(item_gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
    (*item).x40_vel.x = fmadds((*star).accel, (*item).facing_dir, (*item).x40_vel.x);
}

pub unsafe extern "C" fn itYoshistar_UnkMotion0_Coll(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itYoshistar_UnkMotion0_Coll");
    ext::it_8026E0F4(item_gobj);
    false
}

pub unsafe extern "C" fn it_802B3348(item_gobj: *mut Item_GObj, arg1: *mut HSD_GObj) {
    melee_core::ct!("it_802B3348");
    ext::it_8026B894(item_gobj, arg1);
}

// ---- folded from itfwd.rs (consolidation steps 21/22) ----
/// `it_802B3348` (src/melee/it/kinds/ityoshistar.c): `it_8026B894(..)`.
pub fn it_802b3348(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802b3348");
    it_8026b894(ip, ref_gobj);
}
