//! Native port of src/melee/it/kinds/itkirbycutterbeam.c (Kirby's cutter beam). Callees in other
//! TUs go through `ext`; sinf / cosf / atan2f / lbVector_Mirror are the native ports in
//! melee_runtime. Checked against the C (whole TU, shipping contraction) by
//! tests/tu_itkirbycutterbeam.rs.
//! Fused sites (`fmadds`): `itKirbycutterbeam_UnkMotion0_Phys` 1 (`vel.y += 0.05f * vel.y`).
//! The C's discarded `atan2f(normal.x, normal.y)` call in `..._Coll` is a pure function and is
//! not repeated.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

use crate::helpers::{M_PI, M_PI_2, M_TAU};
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_core::vec::Vec3 as RVec;
use melee_runtime::gekko::{cosf, sinf};
use melee_runtime::lbtrigf::atan2f;
use melee_runtime::lbvector;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
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
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjClearFlagsAll(a0, a1) };
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn ftKb_AttackDashAir_800F21C0(a0: *mut HSD_GObj) -> f32;
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026D564(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D564(a0) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_80273130(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273130(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_802750F8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802750F8(a0) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_802762B0(a0: *mut Item) => unsafe { crate::item::it_802762b0(&mut *a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut result: i32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

#[inline]
unsafe fn ip_of(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}
#[inline]
unsafe fn jobj_mtx_is_dirty(jobj: *mut HSD_JObj) -> bool {
    assert!(!jobj.is_null(), "jobj");
    (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
}
#[inline]
unsafe fn jobj_set_mtx_dirty(jobj: *mut HSD_JObj) {
    if !jobj.is_null() && !jobj_mtx_is_dirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}
/// `HSD_JObjSetRotationX` (jobj.h)
#[inline]
unsafe fn jobj_set_rotation_x(jobj: *mut HSD_JObj, x: f32) {
    assert!(!jobj.is_null(), "jobj");
    assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
    (*jobj).rotate.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        jobj_set_mtx_dirty(jobj);
    }
}
/// `HSD_JObjSetRotationY` (jobj.h)
#[inline]
unsafe fn jobj_set_rotation_y(jobj: *mut HSD_JObj, y: f32) {
    assert!(!jobj.is_null(), "jobj");
    assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
    (*jobj).rotate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        jobj_set_mtx_dirty(jobj);
    }
}
/// `Item_InitSpawnCommonFields` (src/melee/it/kinds/inlines.h)
#[inline]
unsafe fn item_init_spawn_common_fields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing_dir: f32, initial_collision: bool) {
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
/// Wrap `angle` into [0, M_TAU] (the C's two `while` loops, f32 variable, double constant).
#[inline]
unsafe fn wrap_tau(angle: *mut f32) {
    while *angle < 0.0 {
        *angle = (*angle as f64 + M_TAU) as f32;
    }
    while *angle as f64 > M_TAU {
        *angle = (*angle as f64 - M_TAU) as f32;
    }
}

/// `ItemStateTable it_803F6798[1]` (3F6798): the C initialiser is the flat `{0, a, p, c}`.
pub static it_803F6798: [ItemStateTable; 1] = [ItemStateTable {
    anim_id: 0,
    animated: Some(itKirbycutterbeam_UnkMotion0_Anim),
    physics_updated: Some(itKirbycutterbeam_UnkMotion0_Phys),
    collided: Some(itKirbycutterbeam_UnkMotion0_Coll),
}];

/// `void it_8029BAB8(HSD_GObj*, Vec3*, float)` (29BAB8)
pub unsafe extern "C" fn it_8029BAB8(gobj: *mut HSD_GObj, pos: *mut Vec3, facing_dir: f32) {
    let mut spawn: SpawnItem = core::mem::zeroed();
    (*pos).z = 0.0;
    spawn.kind = It_Kind_Kirby_CBeam;
    spawn.prev_pos = *pos;
    ext::it_8026BB68(gobj, &mut spawn.pos);
    item_init_spawn_common_fields(&mut spawn, gobj, facing_dir, true);

    let spawned_gobj = ext::Item_80268B18(&mut spawn);
    if !spawned_gobj.is_null() {
        it_8029BB90(spawned_gobj, gobj);
        ext::db_80225DD8(spawned_gobj, gobj);
        ext::it_802750F8(spawned_gobj);
    }
}

/// `void it_8029BB90(Item_GObj*, HSD_GObj*)` (29BB90)
pub unsafe extern "C" fn it_8029BB90(gobj: *mut Item_GObj, owner: *mut HSD_GObj) {
    let ip = ip_of(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itKirbyCutterBeamAttributes;

    (*ip).xDD4_itemVar.kirbycutterbeam.angle = ext::ftKb_AttackDashAir_800F21C0(owner);
    (*ip).xDD4_itemVar.kirbycutterbeam.speed = (*attr).x0_speed;
    (*ip).x40_vel.x = (*ip).facing_dir
        * ((*ip).xDD4_itemVar.kirbycutterbeam.speed * cosf((*ip).xDD4_itemVar.kirbycutterbeam.angle));
    (*ip).x40_vel.y = -(*ip).xDD4_itemVar.kirbycutterbeam.speed * sinf((*ip).xDD4_itemVar.kirbycutterbeam.angle);
    (*ip).x40_vel.z = 0.0;
    ext::it_80275158(gobj, (*attr).x8_lifetime);
    ext::Item_80268E5C(gobj, 0, 2);
    ext::it_802762B0(ip);
    (*ip).xDD4_itemVar.kirbycutterbeam.dir.x = (*ip).facing_dir * sinf((*ip).xDD4_itemVar.kirbycutterbeam.angle);
    (*ip).xDD4_itemVar.kirbycutterbeam.dir.y = cosf((*ip).xDD4_itemVar.kirbycutterbeam.angle);
    (*ip).xDD4_itemVar.kirbycutterbeam.dir.z = 0.0;
    (*ip).xDD4_itemVar.kirbycutterbeam.init_pos = (*ip).pos;
}

/// `bool itKirbycutterbeam_UnkMotion0_Anim(Item_GObj*)` (29BC88)
pub unsafe extern "C" fn itKirbycutterbeam_UnkMotion0_Anim(gobj: *mut Item_GObj) -> i32 {
    ext::it_80273130(gobj)
}

/// `void itKirbycutterbeam_UnkMotion0_Phys(Item_GObj*)` (29BCA8)
pub unsafe extern "C" fn itKirbycutterbeam_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let ip = ip_of(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let child = if jobj.is_null() { null_mut() } else { (*jobj).child };
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itKirbyCutterBeamAttributes;

    (*ip).xDD4_itemVar.kirbycutterbeam.init_pos = (*ip).pos;
    (*ip).x40_vel.x = (*ip).facing_dir
        * ((*ip).xDD4_itemVar.kirbycutterbeam.speed * cosf((*ip).xDD4_itemVar.kirbycutterbeam.angle));
    (*ip).x40_vel.y = -(*ip).xDD4_itemVar.kirbycutterbeam.speed * sinf((*ip).xDD4_itemVar.kirbycutterbeam.angle);
    if (*ip).ground_or_air == GA_Ground && (*ip).x40_vel.y < 0.0 {
        (*ip).x40_vel.y = fmadds(0.05, (*ip).x40_vel.y, (*ip).x40_vel.y);
    }
    (*ip).x40_vel.z = 0.0;
    (*ip).xDD4_itemVar.kirbycutterbeam.speed -= (*attr).xC_decel;
    if (*ip).xDD4_itemVar.kirbycutterbeam.speed < 0.01 {
        (*ip).xDD4_itemVar.kirbycutterbeam.speed = 0.01;
    }
    if (*ip).xD44_lifeTimer <= 5.0 {
        ext::HSD_JObjSetFlagsAll(child, JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    }
}

/// `bool itKirbycutterbeam_UnkMotion0_Coll(Item_GObj*)` (29BDD4)
pub unsafe extern "C" fn itKirbycutterbeam_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    let ip = ip_of(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    if (*ip).ground_or_air == GA_Ground {
        if ext::it_8026D564(gobj) != 0 {
            (*ip).xDD4_itemVar.kirbycutterbeam.angle =
                (*ip).facing_dir * atan2f((*ip).x378_itemColl.floor.normal.x, (*ip).x378_itemColl.floor.normal.y);
        } else {
            ext::it_802762BC(ip);
            (*ip).pos.y += 0.001;
            (*ip).x378_itemColl.last_pos.y += 0.001;
        }
    } else if ext::it_8026DA08(gobj) != 0 {
        ext::it_802762B0(ip);
        (*ip).xDD4_itemVar.kirbycutterbeam.angle =
            (*ip).facing_dir * atan2f((*ip).x378_itemColl.floor.normal.x, (*ip).x378_itemColl.floor.normal.y);
    }

    jobj_set_rotation_x(jobj, (*ip).xDD4_itemVar.kirbycutterbeam.angle);

    if (*ip).x40_vel.x == 0.0 {
        let flags = (*ip).x378_itemColl.env_flags;
        if (flags & Collide_LeftWallMask != 0) || (flags & Collide_RightWallMask != 0) {
            return 1;
        }
    } else if (*ip).x40_vel.x > 0.0 {
        if (*ip).x378_itemColl.env_flags & Collide_LeftWallMask != 0 {
            return 1;
        }
    } else if (*ip).x378_itemColl.env_flags & Collide_RightWallMask != 0 {
        return 1;
    }
    0
}

/// `bool itKirbyCutterBeam_Logic7_DmgDealt(Item_GObj*)` (29BFB4)
pub unsafe extern "C" fn itKirbyCutterBeam_Logic7_DmgDealt(_arg0: *mut Item_GObj) -> i32 {
    0
}

/// `bool itKirbyCutterBeam_Logic7_Clanked(Item_GObj*)` (29BFBC)
pub unsafe extern "C" fn itKirbyCutterBeam_Logic7_Clanked(_arg0: *mut Item_GObj) -> i32 {
    1
}

/// `bool itKirbyCutterBeam_Logic7_Absorbed(Item_GObj*)` (29BFC4)
pub unsafe extern "C" fn itKirbyCutterBeam_Logic7_Absorbed(_arg0: *mut Item_GObj) -> i32 {
    1
}

/// `bool it_2725_Logic7_Reflected(Item_GObj*)` (29BFCC)
pub unsafe extern "C" fn it_2725_Logic7_Reflected(gobj: *mut Item_GObj) -> i32 {
    let ip = ip_of(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    (*ip).facing_dir = -(*ip).facing_dir;
    (*ip).x40_vel.x = -(*ip).x40_vel.x;
    (*ip).x40_vel.y = -(*ip).x40_vel.y;

    if (*ip).ground_or_air == GA_Ground {
        (*ip).xDD4_itemVar.kirbycutterbeam.angle =
            (*ip).facing_dir * atan2f((*ip).x378_itemColl.floor.normal.x, (*ip).x378_itemColl.floor.normal.y);
    } else {
        let base = if (*ip).facing_dir == 1.0 { 0.0 } else { M_PI };
        (*ip).xDD4_itemVar.kirbycutterbeam.angle =
            (base + atan2f((*ip).x40_vel.y, (*ip).x40_vel.x) as f64) as f32;
    }

    wrap_tau(&raw mut (*ip).xDD4_itemVar.kirbycutterbeam.angle);

    jobj_set_rotation_x(jobj, (*ip).xDD4_itemVar.kirbycutterbeam.angle);
    jobj_set_rotation_y(jobj, (M_PI_2 * (*ip).facing_dir as f64) as f32);
    0
}

/// `bool it_2725_Logic7_ShieldBounced(Item_GObj*)` (29C208)
pub unsafe extern "C" fn it_2725_Logic7_ShieldBounced(gobj: *mut Item_GObj) -> i32 {
    let ip = ip_of(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    lbvector::mirror(
        &mut *(&raw mut (*ip).x40_vel as *mut RVec),
        &*(&raw const (*ip).xC58 as *const RVec),
    );
    (*ip).xDD4_itemVar.kirbycutterbeam.angle = atan2f((*ip).x40_vel.y, (*ip).x40_vel.x);

    wrap_tau(&raw mut (*ip).xDD4_itemVar.kirbycutterbeam.angle);

    (*ip).facing_dir = if (*ip).x40_vel.x >= 0.0 { 1.0 } else { -1.0 };

    if (*ip).ground_or_air == GA_Ground {
        (*ip).xDD4_itemVar.kirbycutterbeam.angle =
            (*ip).facing_dir * atan2f((*ip).x378_itemColl.floor.normal.x, (*ip).x378_itemColl.floor.normal.y);
    } else {
        let base = if (*ip).facing_dir == 1.0 { 0.0 } else { M_PI };
        (*ip).xDD4_itemVar.kirbycutterbeam.angle =
            (base + atan2f((*ip).x40_vel.y, (*ip).x40_vel.x) as f64) as f32;
    }

    wrap_tau(&raw mut (*ip).xDD4_itemVar.kirbycutterbeam.angle);

    jobj_set_rotation_x(jobj, (*ip).xDD4_itemVar.kirbycutterbeam.angle);
    jobj_set_rotation_y(jobj, (M_PI_2 * (*ip).facing_dir as f64) as f32);
    0
}

/// `bool itKirbyCutterBeam_Logic7_HitShield(Item_GObj*)` (29C4AC)
pub unsafe extern "C" fn itKirbyCutterBeam_Logic7_HitShield(_arg0: *mut Item_GObj) -> i32 {
    1
}

/// `void it_8029C4B4(Item_GObj*, Item_GObj*)` (29C4B4)
pub unsafe extern "C" fn it_8029C4B4(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
