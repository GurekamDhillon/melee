//! Port of src/melee/it/kinds/itmariofireball.c: Mario's fireball (logic table 87; `it_803F6788`). Callees of
//! unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping
//! contraction) by tests/kinds_itmariofireball.rs.
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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj);
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026D9A0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D9A0(a0) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802750F8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802750F8(a0) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_8027781C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itmaplib::it_8027781C(a0) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::it_3F14::it_804D6D28;
}
// END GENERATED EXT

/// `ItemStateTable it_803F6788[]`
pub static mut it_803F6788: [ItemStateTable; 1] = [st_row(
    0,
    pred_b(itMariofireball_UnkMotion0_Anim),
    Some(itMariofireball_UnkMotion0_Phys),
    pred_b(itMariofireball_UnkMotion0_Coll),
)];

/// `Item_ApplyFallingPhysics` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_ApplyFallingPhysics(gobj: *mut Item_GObj) {
    let attrs = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
}

/// `Item_InitSpawnCommonFields` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing_dir: f32, initial_collision: bool) {
    (*spawn).facing_dir = facing_dir;
    (*spawn).x3C_damage = 0;
    (*spawn).vel.z = 0.0;
    (*spawn).vel.y = 0.0;
    (*spawn).vel.x = 0.0;
    (*spawn).x0_parent_gobj = parent;
    (*spawn).x4_parent_gobj2 = (*spawn).x0_parent_gobj;
    (*spawn).x44_flag.anon_0.set_b0(initial_collision as u8);
    (*spawn).x40 = 0;
}

/// MSL `sqrtf_accurate` (math_ppc.h, static inline): `__frsqrte` plus four Newton steps in double; the
/// shipping build contracts `3.0 - guess*guess*x` into one fma.
#[inline(always)]
fn sqrtf_accurate(x: f32) -> f32 {
    melee_runtime::gekko::sqrtf_accurate(x)
}

/// 29B6F8
pub unsafe extern "C" fn it_8029B6F8(gobj: *mut Item_GObj, pos: *mut Vec3, kind: ItemKind, facing_dir: f32) {
    melee_core::ct!("it_8029B6F8");
    let mut spawn_item: SpawnItem = core::mem::zeroed();
    spawn_item.kind = kind;
    // Item_InitSpawnPositionFromParent(&spawn_item, gobj, pos)
    spawn_item.prev_pos = *pos;
    spawn_item.prev_pos.z = 0.0;
    ext::it_8026BB68(gobj, &mut spawn_item.pos);
    Item_InitSpawnCommonFields(&mut spawn_item, gobj, facing_dir, true);

    let spawned_gobj = ext::Item_80268B18(&mut spawn_item);
    it_8029B7C0(spawned_gobj);
    ext::db_80225DD8(spawned_gobj, gobj);
    ext::it_802750F8(spawned_gobj);
}

/// 29B7C0
pub unsafe extern "C" fn it_8029B7C0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8029B7C0");
    let it = GET_ITEM(gobj);
    let ap = (*(*it).xC4_article_data).x4_specialAttributes as *mut itUnkAttributes;
    let x0 = (*ap).x0_float;
    let x4 = (*ap).x4_float;

    (*it).x40_vel.x = (*it).facing_dir * (x0 * cosf(x4));
    (*it).x40_vel.y = x0 * sinf(x4);
    (*it).x40_vel.z = 0.0;

    ext::it_80275158(gobj, (*ap).x8);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itMariofireball_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMariofireball_UnkMotion0_Anim");
    let item = GET_ITEM(gobj);
    (*item).xD44_lifeTimer -= 1.0;
    (*item).xD44_lifeTimer <= 0.0
}

pub unsafe extern "C" fn itMariofireball_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itMariofireball_UnkMotion0_Phys");
    Item_ApplyFallingPhysics(gobj);
}

/// `calc_dist_2d_accurate` (static): `sqrtf_accurate(SQ(x) + SQ(y))`; `x*x + y*y` contracts to one
/// `fmadds` (x*x fused with the y*y product).
pub unsafe extern "C" fn calc_dist_2d_accurate(v: *mut Vec3) -> f64 {
    melee_core::ct!("calc_dist_2d_accurate");
    sqrtf_accurate(fmadds((*v).x, (*v).x, (*v).y * (*v).y)) as f64
}

pub unsafe extern "C" fn itMariofireball_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMariofireball_UnkMotion0_Coll");
    ext::it_8026D9A0(gobj);
    if ext::it_8027781C(gobj) != 0 {
        let ip = GET_ITEM(gobj);
        let jobj = GET_JOBJ(gobj);
        let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itUnkAttributes;
        if calc_dist_2d_accurate(&mut (*ip).x40_vel) < (*attrs).x10 as f64 {
            return true;
        }
        if (*ip).kind == It_Kind_Mario_Fire {
            ext::Item_8026AE84(ip, !0xFFFD_40C6u32 as i32, 0x7f, 0x40);
            ext::efAsync_Spawn(gobj, &mut (*GET_ITEM(gobj)).xBC0 as *mut _ as *mut c_void, 1, 1147, jobj);
        } else {
            ext::efAsync_Spawn(gobj, &mut (*GET_ITEM(gobj)).xBC0 as *mut _ as *mut c_void, 1, 1184, jobj);
        }
    }
    false
}

pub unsafe extern "C" fn itMarioFireball_Logic87_DmgDealt(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarioFireball_Logic87_DmgDealt");
    true
}

pub unsafe extern "C" fn itMarioFireball_Logic87_Reflected(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarioFireball_Logic87_Reflected");
    ext::it_80273030(gobj) != 0
}

pub unsafe extern "C" fn itMarioFireball_Logic87_Clanked(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarioFireball_Logic87_Clanked");
    true
}

pub unsafe extern "C" fn itMarioFireball_Logic87_HitShield(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarioFireball_Logic87_HitShield");
    true
}

pub unsafe extern "C" fn itMarioFireball_Logic87_Absorbed(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarioFireball_Logic87_Absorbed");
    true
}

pub unsafe extern "C" fn itMarioFireball_Logic87_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMarioFireball_Logic87_ShieldBounced");
    ext::itColl_BounceOffShield(gobj) != 0
}

pub unsafe extern "C" fn itMarioFireball_Logic87_EvtUnk(gobj: *mut Item_GObj, referenced_gobj: *mut HSD_GObj) {
    melee_core::ct!("itMarioFireball_Logic87_EvtUnk");
    ext::it_8026B894(gobj, referenced_gobj);
}
