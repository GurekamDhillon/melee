//! Port of src/melee/it/kinds/itmato.c: Mato (Target Test target, logic table 17). `mato.x0` is the stage joint
//! the target follows. The TARGET_PC branches (scripted targets, pc/gameworld/script_game.c) are ported: they are
//! what the shipping PC build runs. Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked
//! against the C (whole TU, shipping contraction) by tests/kinds_itmato.rs.
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
    fn Ground_801C4338() => unsafe { melee_world::tu::ground::Ground_801C4338() };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn ScriptGame_TargetDestroyed(a0: *mut HSD_GObj) -> i32;
    fn ScriptGame_TargetPin(a0: *mut HSD_GObj) -> i32;
    fn it_8027B730(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_8027b730(&mut *((*a0).user_data as *mut Item)) };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `ItemStateTable it_803F83E0[]`
pub static mut it_803F83E0: [ItemStateTable; 1] = [
    st_row(-1, pred_b(itMato_UnkMotion0_Anim), Some(itMato_UnkMotion0_Phys), pred_b(itMato_UnkMotion0_Coll)),
];

pub unsafe extern "C" fn it_802D84F8(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D84F8");
    let it = GET_ITEM(item_gobj);
    ext::it_8027B730(item_gobj);
    (*it).facing_dir = 0.0;
    (*it).xD5C = 0;
    (*it).xDCC_flag.set_b3(0);
    it_802D8554(item_gobj);
}

pub unsafe extern "C" fn it_802D8554(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D8554");
    let it = GET_ITEM(item_gobj);
    (*it).x40_vel.y = 0.0;
    (*it).x40_vel.x = 0.0;
    ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itMato_UnkMotion0_Anim(_item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMato_UnkMotion0_Anim");
    false
}

pub unsafe extern "C" fn itMato_UnkMotion0_Phys(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itMato_UnkMotion0_Phys");
    let it = GET_ITEM(item_gobj);
    if !(*it).xDD4_itemVar.mato.x0.is_null() {
        let mut pos: Vec3 = core::mem::zeroed();
        ext::lb_8000B1CC((*it).xDD4_itemVar.mato.x0, null_mut(), &mut pos);
        (*it).pos = pos;
    } else {
        // TARGET_PC: a scripted target has no stage joint to follow.
        ext::ScriptGame_TargetPin(item_gobj);
    }
}

pub unsafe extern "C" fn itMato_UnkMotion0_Coll(_item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMato_UnkMotion0_Coll");
    false
}

pub unsafe extern "C" fn it_802D85F4(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802D85F4");
    // TARGET_PC: scripted targets are owned by the Lua stage layer, not Ground_801C4210's counter.
    if ext::ScriptGame_TargetDestroyed(item_gobj) != 0 {
        return true;
    }
    ext::Ground_801C4338();
    true
}
