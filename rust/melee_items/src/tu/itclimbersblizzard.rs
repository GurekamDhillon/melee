//! Port of src/melee/it/kinds/itclimbersblizzard.c: the Ice Climbers' Blizzard shot. Callees of
//! unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping
//! contraction) by tests/kinds_itclimbersblizzard.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::gekko::{cosf, sinf};
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
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026D9A0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D9A0(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_802750F8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802750F8(a0) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_80276308(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80276308(&mut *((*a0).user_data as *mut Item)) };
    fn it_802763E0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::helpers::it_802763e0(&mut *((*a0).user_data as *mut Item)) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

const M_TAU: f64 = 6.283185307179586;
const M_PI_2: f64 = core::f64::consts::FRAC_PI_2;

/// `ItemStateTable it_803F76A8[]`
pub static mut it_803F76A8: [ItemStateTable; 1] = [
    st_row(0, pred_b(itClimbersBlizzard_UnkMotion0_Anim), Some(itClimbersBlizzard_UnkMotion0_Phys), pred_b(itClimbersBlizzard_UnkMotion0_Coll)),
];

/// `Item_TickLifetime` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_TickLifetime(ip: *mut Item) -> bool {
    if (*ip).xD44_lifeTimer <= 0.0 {
        return true;
    }
    (*ip).xD44_lifeTimer -= 1.0;
    false
}

/// `spawn_item_0z` (static inline)
#[inline]
unsafe fn spawn_item_0z(gobj: *mut Item_GObj, kind: ItemKind, pos: *mut Vec3, facing_dir: f32, spawn: *mut SpawnItem) -> *mut Item_GObj {
    (*spawn).kind = kind;
    (*spawn).prev_pos = *pos;
    (*spawn).prev_pos.z = 0.0;
    ext::it_8026BB68(gobj, &mut (*spawn).pos);
    (*spawn).facing_dir = facing_dir;
    (*spawn).x3C_damage = 0;
    (*spawn).vel.z = 0.0;
    (*spawn).vel.y = 0.0;
    (*spawn).vel.x = 0.0;
    (*spawn).x0_parent_gobj = gobj;
    (*spawn).x4_parent_gobj2 = (*spawn).x0_parent_gobj;
    (*spawn).x44_flag.anon_0.set_b0(1);
    (*spawn).x40 = 0;
    ext::Item_80268B18(spawn)
}

pub unsafe extern "C" fn itClimbersBlizzard_Spawn(gobj: *mut Item_GObj, pos: *mut Vec3, facing_dir: f32) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    let spawned = spawn_item_0z(gobj, It_Kind_IceClimber_Blizzard, pos, facing_dir, &mut spawn);
    if !spawned.is_null() {
        let ip = GET_ITEM(spawned);
        (*ip).xDD4_itemVar.climbersblizzard.set_flag0(0);
        let attr = (*ip).xCC_item_attr;
        (*ip).xDD4_itemVar.climbersblizzard.x0 = (*attr).x60_scale;
        itClimbersBlizzard_802C2248(spawned);
        ext::db_80225DD8(spawned, gobj);
        ext::it_8026B3A8(spawned);
        ext::it_802750F8(spawned);
    }
    spawned
}

/// 2C2248: random launch angle inside [xC, x10] (1 fused site: `(x10 - xC) * rand + xC`).
pub unsafe extern "C" fn itClimbersBlizzard_802C2248(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itClimbersBlizzardAttributes;
    let rand = ext::HSD_Randf();
    let mut var_f2 = (*attrs).x10 - (*attrs).xC;
    let temp_f0 = fmadds(var_f2, rand, (*attrs).xC);
    if (*ip).facing_dir == 1.0 {
        var_f2 = temp_f0;
    } else {
        var_f2 = -temp_f0;
    }
    let mut var_f31: f32 = (var_f2 as f64 - M_PI_2) as f32;
    while var_f31 < 0.0 {
        var_f31 = (var_f31 as f64 + M_TAU) as f32;
    }
    while var_f31 as f64 > M_TAU {
        var_f31 = (var_f31 as f64 - M_TAU) as f32;
    }
    (*ip).x40_vel.x = (*attrs).x4 * cosf(var_f31);
    (*ip).x40_vel.y = (*attrs).x4 * sinf(var_f31);
    (*ip).x40_vel.z = 0.0;
    ext::it_80275158(gobj, (*attrs).x0);
    (*ip).xDD4_itemVar.climbersblizzard.set_flag0(1);
    itClimbersBlizzard_802C2358(gobj);
}

/// 2C2358
pub unsafe extern "C" fn itClimbersBlizzard_802C2358(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itClimbersBlizzard_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    Item_TickLifetime(ip)
}

pub unsafe extern "C" fn itClimbersBlizzard_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itClimbersBlizzardAttributes;
    (*ip).x40_vel.y += (*attrs).x8;
}

pub unsafe extern "C" fn itClimbersBlizzard_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    let mut flags: i32 = 0;
    ext::it_8026D9A0(gobj);
    flags |= ext::it_80276308(gobj);
    flags |= ext::it_802763E0(gobj);
    flags != 0
}

pub unsafe extern "C" fn itClimbersBlizzard_DmgDealt(_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itClimbersBlizzard_Reflected(gobj: *mut Item_GObj) -> bool {
    ext::it_80273030(gobj) != 0
}

pub unsafe extern "C" fn itClimbersBlizzard_Clanked(_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itClimbersBlizzard_HitShield(_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itClimbersBlizzard_Absorbed(_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itClimbersBlizzard_ShieldBounced(_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itClimbersBlizzard_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
