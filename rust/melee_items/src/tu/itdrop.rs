//! Port of src/melee/it/itdrop.c: the item-drop helpers (a pick from the stage's item table, an item thrown out of
//! another item - capsule, Pokeball, Mr. Saturn style - and the spawn at a position). Callees of other TUs go
//! through `ext`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::itkinds7::bisect_value;
#[allow(unused_imports)]
use crate::layout::itEgg_ItemVars;
#[allow(unused_imports)]
use crate::layout::itLizardonAttributes;
#[allow(unused_imports)]
use crate::layout::itPeachTurnipAttributes;
#[allow(unused_imports)]
use crate::layout::it_2E5A_Attrs;
#[allow(unused_imports)]
use crate::layout::ItemPickTable;
#[allow(unused_imports)]
use melee_core::fp::frsqrte;
#[allow(unused_imports)]
use melee_runtime::random::HsdRand;
#[allow(unused_imports)]
use crate::helpers::it_link_lerp;
#[allow(unused_imports)]
use crate::itlib::it_80271534;
#[allow(unused_imports)]
use crate::itlib::it_8026b3a8;
#[allow(unused_imports)]
use crate::itlib::it_8026b894;
#[allow(unused_imports)]
use crate::itlib::it_80274d6c;
#[allow(unused_imports)]
use crate::itlib::it_80274ecc;
#[allow(unused_imports)]
use crate::itlib::it_8027572c;
#[allow(unused_imports)]
use crate::itlib3::it_80272860;
#[allow(unused_imports)]
use crate::itlib3::it_80274658;
#[allow(unused_imports)]
use crate::itlib3::it_80274d04;
#[allow(unused_imports)]
use crate::itlib3::it_802756e0;
#[allow(unused_imports)]
use crate::itlib3::it_8027a344;
#[allow(unused_imports)]
use crate::itkinds::itGShell_Attrs;
#[allow(unused_imports)]
use crate::layout::ftCollisionBox;
#[allow(unused_imports)]
use crate::layout::itLinkHookshotAttributes;
#[allow(unused_imports)]
use crate::layout::itMarilAttributes;
#[allow(unused_imports)]
use crate::layout::itSScopeAttributes;
#[allow(unused_imports)]
use crate::layout::itSword_UnkArticle1;
#[allow(unused_imports)]
use crate::layout::itTaruAttributes;
#[allow(unused_imports)]
use crate::layout::itTaruCann_DatAttrs;
#[allow(unused_imports)]
use crate::layout::itToolsAttributes;
#[allow(unused_imports)]
use crate::layout::itUnkAttributes;
#[allow(unused_imports)]
use crate::layout::itWstarAttributes;
#[allow(unused_imports)]
use crate::layout::Collide_LeftWallMask;
#[allow(unused_imports)]
use crate::layout::Collide_RightWallMask;
#[allow(unused_imports)]
use crate::layout::HSD_GObj;
#[allow(unused_imports)]
use crate::layout::HurtCapsule;
#[allow(unused_imports)]
use crate::layout::Item;
#[allow(unused_imports)]
use crate::layout::ItemCommonData;
#[allow(unused_imports)]
use crate::layout::ItemLink;
#[allow(unused_imports)]
use crate::layout::Vec as GcVec;
#[allow(unused_imports)]
use crate::layout::It_Kind_L_Gun_Ray;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use std::ptr::null_mut as null_mut_itkinds6;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/mtu/gen_tu.py; the callees of the C TU)
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
    fn ftLib_IsFighter(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ftlib::ftLib_IsFighter(a0) as _ };
    fn ftLib_800874CC(a0: *mut HSD_GObj, a1: *mut c_void, a2: i32);
    fn gm_8016B0B4() -> i32;
    fn gm_8016B238() -> i32;
    fn it_8026B684(a0: *mut Vec) -> f32 => unsafe { crate::tu::it_26B1::it_8026B684(a0 as _) };
    fn it_8026BB88(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB88(a0, a1 as _) };
    fn it_8026C65C(a0: *mut ItemPickTable) -> i32 => unsafe { crate::tu::itspawn::it_8026C65C(a0) };
    fn it_8026C704() -> i32 => unsafe { crate::tu::itspawn::it_8026C704() as _ };
    fn it_8026C75C(a0: *mut ItemPickTable) -> i32 => unsafe { crate::tu::itspawn::it_8026C75C(a0) };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_80274ED8() => unsafe { crate::tu::it_2725::it_80274ED8() };
    fn it_802E5F00(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: i32) => unsafe { crate::tu::it_2E5A::it_802E5F00(a0 as _, a1 as _, a2 as _, a3) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::item::Item_804A0C64;
    pub use crate::tu::itspawn::it_804A0E50;
    pub use crate::tu::itspawn::it_804A0E60;
    pub use crate::tu::it_3F14::it_804D6D28;
}
// END GENERATED EXT

/// `zero_init[1]` (the C's `0.0f` kept in `.sdata2`).
const zero_init: [f32; 1] = [0.0];

#[inline(always)]
unsafe fn item(gobj: *mut HSD_GObj) -> *mut Item {
    (*gobj).user_data as *mut Item
}

pub unsafe extern "C" fn it_8026F3AC() -> ItemKind {
    melee_core::ct!("it_8026F3AC");
    ext::it_8026C75C(&raw mut ext::it_804A0E50)
}

/// `it_8026F3D4_check_kind` (static inline).
pub unsafe extern "C" fn it_8026F3D4_check_kind(_ip: *mut Item, it_kind: ItemKind) -> i32 {
    melee_core::ct!("it_8026F3D4_check_kind");
    let mut ret = 0;
    if it_kind >= It_Kind_L_Gun_Ray {
        ret = 1;
    }
    ret
}

pub unsafe extern "C" fn it_8026F3D4(item_gobj: *mut HSD_GObj, arg1: *mut it_8026F3D4_arg1_t, num: i32, arg3: i32) -> i32 {
    melee_core::ct!("it_8026F3D4");
    let mut sp30: [ItemKind; 8] = [0; 8];
    let mut sp24 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut spawned_item_gobj: *mut HSD_GObj = null_mut();
    let mut chk1 = 0i32;
    let mut it_kind: ItemKind = -1;
    let new_var = arg3 & 1;
    let item = item(item_gobj);
    let mut cnt2 = 0i32;
    let zero = zero_init[0];
    let mut cnt = 0;
    while cnt < num {
        if new_var == 0 || it_kind == -1 {
            it_kind = ext::it_8026C75C(&raw mut ext::it_804A0E50);
        }
        if it_kind == -1 || it_kind >= It_Kind_L_Gun_Ray || (it_kind == It_Kind_M_Ball && ext::it_8026C704() != 0) {
            chk1 |= it_8026F3D4_check_kind(item, it_kind);
            break;
        }
        spawned_item_gobj = it_8026F5C8(item_gobj, it_kind, &raw mut (*item).pos);
        if !spawned_item_gobj.is_null() {
            sp24.x = zero;
            sp24.y = zero;
            sp24.z = zero;
            it_8026F53C(spawned_item_gobj, &raw mut sp24, 0);
            chk1 = 0;
            ext::it_80274ED8();
            sp30[cnt2 as usize] = it_kind;
            cnt2 += 1;
        }
        cnt += 1;
    }
    if ext::ftLib_IsFighter((*item).owner) != 0 {
        let n = cnt2;
        if n != 0 {
            ext::ftLib_800874CC((*item).owner, sp30.as_mut_ptr() as *mut c_void, n);
        }
    }
    if !arg1.is_null() {
        (*arg1).x0 = chk1;
        (*arg1).x4 = it_kind;
        (*arg1).x8 = spawned_item_gobj;
    }
    chk1
}

pub unsafe extern "C" fn it_8026F53C(item_gobj: *mut HSD_GObj, vel: *mut Vec3, chk: i32) {
    melee_core::ct!("it_8026F53C");
    let item = item(item_gobj);
    if chk == 0 {
        (*vel).x = (*ext::it_804D6D28).x54_float * (2.0 * (ext::HSD_Randf() - 0.5));
        (*vel).y = (*(*item).xCC_item_attr).x18;
        (*vel).z = zero_init[0];
    }
    (*item).x40_vel = *vel;
}

pub unsafe extern "C" fn it_8026F5C8(item_gobj: *mut HSD_GObj, kind: ItemKind, pos: *mut Vec3) -> *mut HSD_GObj {
    melee_core::ct!("it_8026F5C8");
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = kind;
    spawn.vel.z = zero_init[0];
    spawn.vel.y = zero_init[0];
    spawn.vel.x = zero_init[0];
    spawn.x3C_damage = 0;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.prev_pos = *pos;
    spawn.prev_pos.z = zero_init[0];
    spawn.x40 = 0;
    spawn.facing_dir = ext::it_8026B684(&raw mut spawn.prev_pos);
    if !item_gobj.is_null() {
        ext::it_8026BB88(item_gobj, &raw mut spawn.pos);
        spawn.x0_parent_gobj = null_mut();
        spawn.x4_parent_gobj2 = item_gobj;
    } else {
        spawn.pos = spawn.prev_pos;
        spawn.x0_parent_gobj = null_mut();
        spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    }
    let spawn_gobj = ext::Item_80268B18(&raw mut spawn);
    if !spawn_gobj.is_null() {
        ext::it_80274658(spawn_gobj, (*ext::it_804D6D28).x70_float);
    }
    spawn_gobj
}

pub unsafe extern "C" fn it_8026F6BC(item_gobj: *mut HSD_GObj, arg1: *mut Vec3, vel: *mut Vec3, chk: i32) -> *mut HSD_GObj {
    melee_core::ct!("it_8026F6BC");
    if ext::Item_804A0C64.x2C >= ext::Item_804A0C64.x30 {
        return null_mut();
    }
    if ext::HSD_Randi((*ext::it_804D6D28).x124) != 0 {
        return null_mut();
    }
    // TARGET_PC: the same empty-pick-table guard as itspawn.c
    if ext::it_804A0E60.size == 0 || ext::it_804A0E60.x4.is_null() || ext::it_804A0E60.xC.is_null() {
        return null_mut();
    }
    let spawned_item_gobj = it_8026F5C8(item_gobj, ext::it_8026C65C(&raw mut ext::it_804A0E60), arg1);
    if !spawned_item_gobj.is_null() {
        let spawned_item = item(spawned_item_gobj);
        if chk == 0 {
            (*vel).x = (*ext::it_804D6D28).x54_float * (2.0 * (ext::HSD_Randf() - 0.5));
            (*vel).y = (*(*spawned_item).xCC_item_attr).x18;
            (*vel).z = zero_init[0];
        }
        (*spawned_item).x40_vel = *vel;
    }
    spawned_item_gobj
}

pub unsafe extern "C" fn it_8026F7C8(arg0: *mut Vec3, vel: *mut Vec3, chk: i32) -> *mut HSD_GObj {
    melee_core::ct!("it_8026F7C8");
    if ext::gm_8016B238() != 0 {
        return null_mut();
    }
    let it_kind = ext::it_8026C75C(&raw mut ext::it_804A0E50);
    if it_kind == -1 {
        return null_mut();
    }
    let spawned_item_gobj = it_8026F5C8(null_mut(), it_kind, arg0);
    if !spawned_item_gobj.is_null() {
        let spawned_item = item(spawned_item_gobj);
        if chk == 0 {
            (*vel).x = (*ext::it_804D6D28).x54_float * (2.0 * (ext::HSD_Randf() - 0.5));
            (*vel).y = (*(*spawned_item).xCC_item_attr).x18;
            (*vel).z = zero_init[0];
        }
        (*spawned_item).x40_vel = *vel;
        ext::it_80274ED8();
    }
    spawned_item_gobj
}

pub unsafe extern "C" fn it_8026F8B4(item_gobj: *mut HSD_GObj, arg1: *mut Vec3, arg2: *mut Vec3, chk: i32) -> i32 {
    melee_core::ct!("it_8026F8B4");
    if ext::gm_8016B0B4() == 0 {
        return 0;
    }
    if ext::HSD_Randi((*ext::it_804D6D28).x138) != 0 {
        return 0;
    }
    let mut rand_int = ext::HSD_Randi((*ext::it_804D6D28).x13C);
    rand_int = rand_int.wrapping_add((*ext::it_804D6D28).x140);
    if chk == 0 {
        (*arg2).x = (*ext::it_804D6D28).x54_float * (2.0 * (ext::HSD_Randf() - 0.5));
        (*arg2).y = (*ext::it_804D6D28).x144;
        (*arg2).z = zero_init[0];
    }
    ext::it_802E5F00(item_gobj, arg1, arg2, rand_int);
    1
}

// ---- folded from itkinds6.rs (consolidation steps 21/22) ----
/// `it_8026F3D4_check_kind` (itdrop.c, static inline): `kind >= It_Kind_L_Gun_Ray`.
pub fn it_8026f3d4_check_kind(_ip: Option<&Item>, it_kind: i32) -> bool {
    melee_core::ct!("it_8026f3d4_check_kind");
    it_kind >= It_Kind_L_Gun_Ray
}

// ---- folded from itkinds9.rs (consolidation steps 21/22) ----
/// `it_8026F53C` (itdrop.c): unless `chk`, draws the drop velocity; then copies it to the item.
/// `common` is the C global `it_804D6D28`.
/// # Safety
/// `ip.xCC_item_attr` is valid when `chk` is false.
pub unsafe fn it_8026f53c(rand: &mut HsdRand, ip: &mut Item, vel: &mut GcVec, chk: bool, common: &ItemCommonData) {
    melee_core::ct!("it_8026f53c");
    if !chk {
        vel.x = common.x54_float * (2.0 * (rand.randf() - 0.5));
        vel.y = (*ip.xCC_item_attr).x18;
        vel.z = 0.0;
    }
    ip.x40_vel = *vel;
}
