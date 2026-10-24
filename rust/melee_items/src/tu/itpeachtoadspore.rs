//! Port of src/melee/it/kinds/itpeachtoadspore.c: the spore cloud Toad releases (logic tables 68 and 92).
//! Fused sites (checked in the C): `min_speed + max_speed_offset * rand` and `0.5F * double_angle +
//! angle_offset` are `fmadds`. `sinf` / `cosf` are the gekko ports. Callees of unported TUs go through
//! `ext` (unset: panics "TODO"). Checked against the C (whole TU) by tests/kinds_itpeachtoadspore.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
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
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn efLib_DestroyAll(a0: *mut HSD_GObj);
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj) -> *mut c_void;
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_80272A3C(a0: *mut HSD_JObj) => unsafe { crate::tu::it_2725::it_80272A3C(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `ItemStateTable it_803F7548[]`
pub static mut it_803F7548: [ItemStateTable; 1] = [
    st_row(0, pred_b(itPeachtoadspore_UnkMotion0_Anim), Some(itPeachtoadspore_UnkMotion0_Phys), None),
];

/// 2BE214
pub unsafe extern "C" fn it_802BE214(item_gobj: *mut Item_GObj, pos: *mut Vec3, kind: ItemKind, facing_dir: f32) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = kind;
    // Item_InitSpawnPosition(&spawn, pos, false)
    spawn.prev_pos = *pos;
    spawn.pos = spawn.prev_pos;
    // Item_InitSpawnCommonFields(&spawn, item_gobj, facing_dir, true)
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = item_gobj;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;

    let spawn_gobj = ext::Item_80268B18(&mut spawn);
    if !spawn_gobj.is_null() {
        it_802BE2E8(spawn_gobj, item_gobj);
        ext::db_80225DD8(spawn_gobj, item_gobj);
        return spawn_gobj;
    }
    null_mut()
}

/// 2BE2E8
pub unsafe extern "C" fn it_802BE2E8(item_gobj: *mut Item_GObj, arg1: *mut HSD_GObj) {
    let item_1 = GET_ITEM(item_gobj);
    let item_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    let attr = (*(*item_1).xC4_article_data).x4_specialAttributes as *mut itPeachToadSporeAttributes;
    (*item_1).owner = arg1;
    ext::it_80275158(item_gobj, 60.0);
    (*item_1).xDAC_itcmd_var0 = 0;

    let mut rand = ext::HSD_Randf();
    let speed = fmadds((*attr).x4_max_speed_offset, rand, (*attr).x0_min_speed);

    rand = ext::HSD_Randf();
    let double_angle = (core::f64::consts::PI as f32) - (*attr).xc_angle;
    let angle_offset = (*attr).xc_angle * rand;
    let angle = fmadds(0.5, double_angle, angle_offset);

    (*item_1).x40_vel.x = (*item_1).facing_dir * (speed * sinf(angle));
    (*item_1).x40_vel.y = speed * cosf(angle);

    ext::it_80272A3C(item_jobj);
    ext::it_8026B3A8(item_gobj);

    let item_2 = GET_ITEM(item_gobj);
    let hsd_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    if (*item_2).kind == It_Kind_Peach_ToadSpore {
        ext::efSync_Spawn(0x4D3, item_gobj, hsd_jobj);
    } else {
        ext::efSync_Spawn(0x4D3, item_gobj, hsd_jobj);
    }
    ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itPeachtoadspore_UnkMotion0_Anim(item_gobj: *mut Item_GObj) -> bool {
    let item = GET_ITEM(item_gobj);
    if (*item).xDAC_itcmd_var0 != 0 {
        return true;
    }

    (*item).xD44_lifeTimer -= 1.0;
    if (*item).xD44_lifeTimer <= 0.0 {
        (*item).xD44_lifeTimer = 0.0;
        return true;
    }
    false
}

pub unsafe extern "C" fn itPeachtoadspore_UnkMotion0_Phys(item_gobj: *mut Item_GObj) {
    let item = GET_ITEM(item_gobj);
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itPeachToadSporeAttributes;
    (*item).x40_vel.x *= (*attr).x8_speed_decay_rate;
    (*item).x40_vel.y *= (*attr).x8_speed_decay_rate;
}

pub unsafe extern "C" fn itPeachToadSpore_Logic92_DmgDealt(item_gobj: *mut Item_GObj) -> bool {
    ext::efLib_DestroyAll(item_gobj);
    true
}

pub unsafe extern "C" fn itPeachToadSpore_Logic68_Clanked(item_gobj: *mut Item_GObj) -> bool {
    ext::efLib_DestroyAll(item_gobj);
    true
}

pub unsafe extern "C" fn itPeachToadSpore_Logic68_HitShield(item_gobj: *mut Item_GObj) -> bool {
    ext::efLib_DestroyAll(item_gobj);
    true
}

pub unsafe extern "C" fn itPeachToadSpore_Logic68_Absorbed(item_gobj: *mut Item_GObj) -> bool {
    ext::efLib_DestroyAll(item_gobj);
    true
}

pub unsafe extern "C" fn itPeachToadSpore_Logic68_ShieldBounced(item_gobj: *mut Item_GObj) -> bool {
    ext::itColl_BounceOffShield(item_gobj) != 0
}

pub unsafe extern "C" fn itPeachToadSpore_Logic68_Reflected(item_gobj: *mut Item_GObj) -> bool {
    let item = GET_ITEM(item_gobj);
    let life_timer = (*item).xD44_lifeTimer;
    ext::it_80273030(item_gobj);
    (*item).xD44_lifeTimer = life_timer;
    false
}

pub unsafe extern "C" fn itPeachToadSpore_Logic92_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
