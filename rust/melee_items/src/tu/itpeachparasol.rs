//! Port of src/melee/it/kinds/itpeachparasol.c: Peach's parasol (logic table 60, the up-B umbrella
//! that follows the fighter). Callees of unported TUs go through `ext` (unset: panics "TODO").
//! Checked against the C (whole TU, shipping contraction) by tests/kinds_itpeachparasol.rs.
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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_802694CC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_802694CC(a0) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn ftPe_8011D518(a0: *mut HSD_GObj) -> i32;
    fn ftPe_SpecialHi_NotActive(a0: *mut HSD_GObj) -> i32;
    fn it_8026B724(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b724(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B73C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b73c(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `ItemStateTable it_803F74F8[]`
pub static mut it_803F74F8: [ItemStateTable; 3] = [
    st_row(-1, pred_b(itPeachparasol_UnkMotion2_Anim), None, None),
    st_row(0, pred_b(itPeachparasol_UnkMotion2_Anim), None, None),
    st_row(1, pred_b(itPeachparasol_UnkMotion2_Anim), None, None),
];

/// `static int it_804D5518[] = { 15, 16 }` (function-local in the C). The C indexes it with the
/// state row's `anim_id` (-1, 0 or 1); row -1 would read the word in front of the array on the
/// console (unknown without the disc), the port returns 0 there and the oracle never asks for it.
static it_804D5518: [i32; 2] = [15, 16];

/// 2BDA40
pub unsafe extern "C" fn it_802BDA40(_item_gobj: *mut Item_GObj, i: i32) -> i32 {
    melee_core::ct!("it_802BDA40");
    let row = (it_803F74F8.as_ptr()).offset(i as isize);
    let anim_id = (*row).anim_id;
    if anim_id < 0 || anim_id as usize >= it_804D5518.len() {
        return 0;
    }
    it_804D5518[anim_id as usize]
}

/// 2BDA64
pub unsafe extern "C" fn it_802BDA64(parent: *mut HSD_GObj, pos: *mut Vec3, arg2: u32, dir: f32) -> *mut Item_GObj {
    melee_core::ct!("it_802BDA64");
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = It_Kind_Peach_Parasol;
    // Item_InitSpawnPosition(&spawn, pos, false)
    spawn.prev_pos = *pos;
    spawn.pos = spawn.prev_pos;
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
        // Item_AttachToParent(item_gobj, parent, arg2)
        ext::Item_8026AB54(item_gobj, parent, arg2);
        ext::db_80225DD8(item_gobj, parent);
        return item_gobj;
    }
    null_mut()
}

pub unsafe extern "C" fn itPeachParasol_Logic60_Destroyed(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itPeachParasol_Logic60_Destroyed");
    let item = GET_ITEM(item_gobj);
    if !(*item).owner.is_null() && ext::ftPe_8011D518((*item).owner) != 0 {
        (*item).owner = null_mut();
    }
}

/// 2BDB94
pub unsafe extern "C" fn it_802BDB94(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BDB94");
    let item = GET_ITEM(item_gobj);
    if !item.is_null() {
        itPeachParasol_Logic60_Destroyed(item_gobj);
        ext::Item_8026A8EC(item_gobj);
    }
}

/// 2BDBF8
pub unsafe extern "C" fn it_802BDBF8(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BDBF8");
    ext::it_8026B724(item_gobj);
}

/// 2BDC18
pub unsafe extern "C" fn it_802BDC18(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BDC18");
    ext::it_8026B73C(item_gobj);
}

/// 2BDC38
pub unsafe extern "C" fn it_802BDC38(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802BDC38");
    let item = GET_ITEM(item_gobj);
    if (*item).msid != 2 {
        return false;
    }
    if (*item).x5CC_currentAnimFrame >= 16.0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn itPeachParasol_Logic60_PickedUp(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itPeachParasol_Logic60_PickedUp");
    let item = GET_ITEM(item_gobj);
    it_802BDDB4(item_gobj, 16.0);
    ext::Item_802694CC(item_gobj);
    (*item).x5D0_animFrameSpeed = 1.0;
}

pub unsafe extern "C" fn itPeachparasol_UnkMotion2_Anim(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itPeachparasol_UnkMotion2_Anim");
    let item = GET_ITEM(item_gobj);
    let var_r3: bool = if !(*item).owner.is_null() {
        ext::ftPe_SpecialHi_NotActive((*item).owner) != 0
    } else {
        true
    };
    if var_r3 {
        itPeachParasol_Logic60_Destroyed(item_gobj);
        return true;
    }
    false
}

/// 2BDD3C
pub unsafe extern "C" fn it_802BDD3C(_item_gobj: *mut Item_GObj) { melee_core::ct!("it_802BDD3C");}

/// 2BDD40
pub unsafe extern "C" fn it_802BDD40(item_gobj: *mut Item_GObj, arg8: f32) {
    melee_core::ct!("it_802BDD40");
    let item = GET_ITEM(item_gobj);
    ext::Item_8026AE84(item, 0xF7, 0x7F, 0x40);
    (*item).x5D0_animFrameSpeed = arg8;
    ext::Item_80268E5C(item_gobj, 1, ITEM_ANIM_UPDATE);
    (*item).on_accessory = Some(it_802BDD3C);
}

/// 2BDDB4
pub unsafe extern "C" fn it_802BDDB4(item_gobj: *mut Item_GObj, arg8: f32) {
    melee_core::ct!("it_802BDDB4");
    let item = GET_ITEM(item_gobj);
    (*item).x5D0_animFrameSpeed = arg8;
    ext::Item_80268E5C(item_gobj, 2, ITEM_ANIM_UPDATE);
    (*item).on_accessory = Some(it_802BDD3C);
}

pub unsafe extern "C" fn itPeachParasol_Logic60_EvtUnk(item_gobj: *mut Item_GObj, arg1: *mut HSD_GObj) {
    melee_core::ct!("itPeachParasol_Logic60_EvtUnk");
    ext::it_8026B894(item_gobj, arg1);
}
