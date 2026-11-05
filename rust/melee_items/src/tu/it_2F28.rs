//! Port of src/melee/it/kinds/it_2F28.c: the unnamed `It_Kind_Unk2` item (a decelerating timed projectile spawned
//! with a velocity and a deceleration `unk2.x0`). `sqrtf` is the MSL `math_ppc.h` one
//! (`melee_runtime::gekko::sqrtf`). Callees of unported TUs go through `ext` (unset: panics "TODO").
//! Checked against the C (whole TU, shipping contraction) by tests/kinds_it_2F28.rs.
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
use melee_runtime::gekko::sqrtf;
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
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026DFB0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DFB0(a0) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_802758D4(a0: *mut HSD_GObj) -> f32 => unsafe { crate::itlib::it_802758d4(&*((*a0).user_data as *mut Item)) };
    fn it_8027B378(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: f32) => unsafe { crate::tu::it_279C::it_8027B378(a0, a1, a2) };
    fn it_8027B4A4(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_8027B4A4(a0, a1) };
    fn it_8027CE44(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CE44(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `ItemStateTable it_803F9470[]`
pub static mut it_803F9470: [ItemStateTable; 1] = [
    st_row(0, pred_b!(it_2F28_UnkMotion0_Anim), Some(it_2F28_UnkMotion0_Phys), pred_b!(it_2F28_UnkMotion0_Coll)),
];

/// `Item_InitSpawnPosition` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec3, on_plane: bool) {
    (*spawn).prev_pos = *pos;
    if on_plane {
        (*spawn).prev_pos.z = 0.0;
    }
    (*spawn).pos = (*spawn).prev_pos;
}

/// 2F28C8
pub unsafe extern "C" fn it_802F28C8(item_gobj: *mut Item_GObj, arg1: i32, arg2: f32) {
    melee_core::ct!("it_802F28C8");
    let item = (*item_gobj).user_data as *mut Item;
    ext::it_8027B4A4((*item).grab_victim, item_gobj);
    ext::it_8027B378((*item).grab_victim, item_gobj, arg2);
    if arg1 != 0 {
        (*item).destroy_type = 0;
    }
    (*item).atk_victim = null_mut();
    (*item).grab_victim = null_mut();
    ext::it_8027CE44(item_gobj);
    ext::Item_8026A8EC(item_gobj);
}

/// 2F295C: spawns the item; returns `it_802758D4` of it (0 when the spawn failed).
pub unsafe extern "C" fn it_802F295C(owner_gobj: *mut HSD_GObj, attr: *mut itUnk2_DatAttrs) -> f32 {
    melee_core::ct!("it_802F295C");
    let mut spawn: SpawnItem = core::mem::zeroed();
    let mut var_f31: f32 = 0.0;
    spawn.kind = It_Kind_Unk2;
    Item_InitSpawnPosition(&mut spawn, &mut (*attr).pos, true);
    spawn.facing_dir = -1.0;
    spawn.x3C_damage = 0;
    spawn.vel = (*attr).vel;
    spawn.x0_parent_gobj = owner_gobj;
    spawn.x4_parent_gobj2 = owner_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;
    let item_gobj = ext::Item_80268B18(&mut spawn);

    if !item_gobj.is_null() {
        let item = (*item_gobj).user_data as *mut Item;
        (*item).xDD4_itemVar.unk2.x0 = (*attr).float2;
        ext::it_80275158(item_gobj, (*attr).float1);
        it_802F2A58(item_gobj);
        var_f31 = ext::it_802758D4(item_gobj);
    }
    var_f31
}

/// `static void it_802F2A58`
pub unsafe extern "C" fn it_802F2A58(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802F2A58");
    let item = (*item_gobj).user_data as *mut Item;
    ext::it_8026B3A8(item_gobj);
    ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
    (*item).xDCE_flag.anon_0.set_b7(0);
}

/// `static bool it_2F28_UnkMotion0_Anim`
pub unsafe extern "C" fn it_2F28_UnkMotion0_Anim(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2F28_UnkMotion0_Anim");
    let item = (*item_gobj).user_data as *mut Item;
    (*item).xD44_lifeTimer = (*item).xD44_lifeTimer - 1.0;
    (*item).xD44_lifeTimer <= 0.0
}

/// `static void it_2F28_UnkMotion0_Phys`: sheds `unk2.x0` of speed per frame (x only, when slower: stops x).
pub unsafe extern "C" fn it_2F28_UnkMotion0_Phys(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_2F28_UnkMotion0_Phys");
    let item = GET_ITEM(item_gobj);
    // SQ(x) + SQ(y): the left product is fused
    // SQ(item->x40_vel.x) + ...: not fused on the console (notes/fp-audit.md)
    let var_f4 = sqrtf((*item).x40_vel.x * (*item).x40_vel.x + (*item).x40_vel.y * (*item).x40_vel.y);
    if var_f4 > (*item).xDD4_itemVar.unk2.x0 {
        (*item).x40_vel.x = ((*item).x40_vel.x * (var_f4 - (*item).xDD4_itemVar.unk2.x0)) / var_f4;
        (*item).x40_vel.y = ((*item).x40_vel.y * (var_f4 - (*item).xDD4_itemVar.unk2.x0)) / var_f4;
    } else {
        (*item).x40_vel.x = 0.0;
    }
}

/// `static bool it_2F28_UnkMotion0_Coll`
pub unsafe extern "C" fn it_2F28_UnkMotion0_Coll(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2F28_UnkMotion0_Coll");
    ext::it_8026DFB0(item_gobj) != 0
}

/// 2F2BDC
pub unsafe extern "C" fn it_802F2BDC(item_gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802F2BDC");
    ext::it_8026B894(item_gobj, ref_gobj);
}

// ---- folded from itfwd.rs (consolidation steps 21/22) ----
/// `it_802F2BDC` (src/melee/it/kinds/it_2F28.c): `it_8026B894(..)`.
pub fn it_802f2bdc(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802f2bdc");
    it_8026b894(ip, ref_gobj);
}
