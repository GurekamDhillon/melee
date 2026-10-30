//! Native port of src/melee/it/kinds/itfoxlaser.c: Fox / Falco's laser shot item (the "ray" item family):
//! spawn from the fighter, per-frame velocity from `speed` / `angle`, the stretching scale, the
//! collision probe, reflect and shield-bounce. Callees in other TUs go through `ext` (unported: panics
//! "TODO"); the header inlines the TU uses (`Item_UpdateRayAnimation`, `Item_BounceRayOffShield`,
//! `Item_ResetRayAfterReflection`, `Item_NormalizeAngle`, the spawn helpers, the jobj.h accessors) are
//! ported here. `cosf` / `sinf` are the MSL functions of melee_runtime::gekko (the oracle links the real
//! C ones); `atan2f` stays an `ext` hook. Checked against the C (the whole TU, shipping contraction)
//! by tests/tu_itfoxlaser.rs.
//!
//! No fused multiply-add sites. `M_PI`, `M_PI_2` and `M_TAU` are doubles: the angle arithmetic of the
//! inlines (`*angle += M_TAU`, `M_PI + atan2f(..)`, `M_PI_2 * facing_dir`) runs in f64 and narrows on store.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_runtime::gekko::{cosf, sinf};
use melee_types::inline as ji;
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026E9A4(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026E9A4(a0, a1 as _, a2 as _, a3 as _) };
    fn it_80273130(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273130(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn lbVector_Mirror(a0: *mut Vec, a1: *mut Vec) => unsafe { melee_runtime::tu::lbvector::lbVector_Mirror(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

const M_PI: f64 = 3.14159265358979323846;
const M_PI_2: f64 = M_PI / 2.0;
const M_TAU: f64 = 6.283185307179586;

/// jobj.h's `HSD_JObjSetMtxDirty` macro / out-of-line dirty marks.
struct Dirty;
impl ji::JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) {
        if !ji::HSD_JObjMtxIsDirty(j) {
            ext::HSD_JObjSetMtxDirtySub(j);
        }
    }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}

/// `HSD_ASSERT(line, jobj)` of the jobj.h accessors: a NULL jobj hangs the console.
#[inline]
unsafe fn jref<'a>(j: *mut HSD_JObj) -> &'a mut HSD_JObj {
    assert!(!j.is_null(), "jobj.h: HSD_ASSERT(jobj)");
    &mut *j
}

#[inline]
unsafe fn item_of(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

#[inline]
unsafe fn jobj_of(g: *mut HSD_GObj) -> *mut HSD_JObj {
    (*g).hsd_obj as *mut HSD_JObj
}

/// `ABS(x)`: `(x) < 0 ? -(x) : (x)` (keeps the sign of a zero).
#[inline]
fn c_abs(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

/// `Item_InitSpawnCommonFields` (itkinds inlines.h).
#[inline]
pub unsafe fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing_dir: f32, initial_collision: bool) {
    melee_core::ct!("Item_InitSpawnCommonFields");
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

/// `Item_InitSpawnPositionFromParent` (itkinds inlines.h).
#[inline]
pub unsafe fn Item_InitSpawnPositionFromParent(spawn: *mut SpawnItem, parent: *mut HSD_GObj, pos: *const Vec) {
    melee_core::ct!("Item_InitSpawnPositionFromParent");
    (*spawn).prev_pos = *pos;
    (*spawn).prev_pos.z = 0.0;
    ext::it_8026BB68(parent, &mut (*spawn).pos);
}

/// `Item_NormalizeAngle` (itkinds inlines.h): wraps into [0, M_TAU] (the arithmetic is in doubles).
#[inline]
pub unsafe fn Item_NormalizeAngle(angle: *mut f32) {
    melee_core::ct!("Item_NormalizeAngle");
    while *angle < 0.0 {
        *angle = (*angle as f64 + M_TAU) as f32;
    }
    while (*angle as f64) > M_TAU {
        *angle = (*angle as f64 - M_TAU) as f32;
    }
}

/// `Item_UpdateRayAnimation` (itkinds inlines.h): velocity from speed / angle, facing, the jobj rotation
/// and the growing scale.
pub unsafe fn Item_UpdateRayAnimation(gobj: *mut Item_GObj, ip: *mut Item, jobj: *mut HSD_JObj, max_scale: *const f32, scale_divisor: f32) -> i32 {
    melee_core::ct!("Item_UpdateRayAnimation");
    (*ip).x40_vel.x = (*ip).xDD4_itemVar.ray.speed * cosf((*ip).xDD4_itemVar.ray.angle);
    (*ip).x40_vel.y = (*ip).xDD4_itemVar.ray.speed * sinf((*ip).xDD4_itemVar.ray.angle);
    (*ip).x40_vel.z = 0.0;
    let dir = if (*ip).x40_vel.x > 0.0 { 1.0 } else { -1.0 };
    (*ip).facing_dir = dir;
    ji::HSD_JObjSetRotationY(jref(jobj), (M_PI_2 * (*ip).facing_dir as f64) as f32, &mut Dirty);
    let vel_x = if (*ip).facing_dir == 1.0 { -(*ip).x40_vel.x } else { (*ip).x40_vel.x };
    let at = ext::atan2f((*ip).x40_vel.y, vel_x);
    ji::HSD_JObjSetRotationX(jref(jobj), (M_PI + at as f64) as f32, &mut Dirty);
    (*ip).xDD4_itemVar.ray.scale += c_abs((*ip).xDD4_itemVar.ray.speed) / scale_divisor;
    if (*ip).xDD4_itemVar.ray.scale > *max_scale {
        (*ip).xDD4_itemVar.ray.scale = *max_scale;
    }
    if (*ip).xDD4_itemVar.ray.scale < 1e-5 {
        (*ip).xDD4_itemVar.ray.scale = 1e-3;
    }
    ji::HSD_JObjSetScaleZ(jref(jobj), (*ip).xDD4_itemVar.ray.scale, &mut Dirty);
    ext::it_80273130(gobj)
}

/// `Item_BounceRayOffShield` (itkinds inlines.h).
pub unsafe fn Item_BounceRayOffShield(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("Item_BounceRayOffShield");
    let ip = item_of(gobj);
    ext::lbVector_Mirror(&mut (*ip).x40_vel, &mut (*ip).xC58);
    (*ip).xDD4_itemVar.ray.scale = 1e-3;
    (*ip).xDD4_itemVar.ray.angle = ext::atan2f((*ip).x40_vel.y, (*ip).x40_vel.x);
    while (*ip).xDD4_itemVar.ray.angle < 0.0 {
        (*ip).xDD4_itemVar.ray.angle = ((*ip).xDD4_itemVar.ray.angle as f64 + M_TAU) as f32;
    }
    while ((*ip).xDD4_itemVar.ray.angle as f64) > M_TAU {
        (*ip).xDD4_itemVar.ray.angle = ((*ip).xDD4_itemVar.ray.angle as f64 - M_TAU) as f32;
    }
    0
}

/// `Item_ResetRayAfterReflection` (itkinds inlines.h).
pub unsafe fn Item_ResetRayAfterReflection(ip: *mut Item, jobj: *mut HSD_JObj) {
    melee_core::ct!("Item_ResetRayAfterReflection");
    (*ip).xDD4_itemVar.ray.scale = 1e-3;
    ji::HSD_JObjSetScaleZ(jref(jobj), (*ip).xDD4_itemVar.ray.scale, &mut Dirty);
    (*ip).xDD4_itemVar.ray.angle = ((*ip).xDD4_itemVar.ray.angle as f64 + M_PI) as f32;
    while (*ip).xDD4_itemVar.ray.angle < 0.0 {
        (*ip).xDD4_itemVar.ray.angle = ((*ip).xDD4_itemVar.ray.angle as f64 + M_TAU) as f32;
    }
    while ((*ip).xDD4_itemVar.ray.angle as f64) > M_TAU {
        (*ip).xDD4_itemVar.ray.angle = ((*ip).xDD4_itemVar.ray.angle as f64 - M_TAU) as f32;
    }
}

/// `ItemStateTable it_803F67D0[2]`.
pub static it_803F67D0: [ItemStateTable; 2] = [
    ItemStateTable { anim_id: 0, animated: Some(itFoxlaser_UnkMotion1_Anim), physics_updated: Some(itFoxlaser_UnkMotion1_Phys), collided: Some(itFoxlaser_UnkMotion1_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itFoxlaser_UnkMotion1_Anim), physics_updated: Some(itFoxlaser_UnkMotion1_Phys), collided: Some(itFoxlaser_UnkMotion1_Coll) },
];

/// `bool it_8029C4D4(Item_GObj*, CollData*)`: the shot's segment-vs-map probe from the previous to the current position.
pub unsafe extern "C" fn it_8029C4D4(item_gobj: *mut Item_GObj, arg1: *mut CollData) -> i32 {
    melee_core::ct!("it_8029C4D4");
    let item = item_of(item_gobj);
    ext::it_8026E9A4(item_gobj, &mut (*item).xDD4_itemVar.foxlaser.pos, &mut (*item).pos, null_mut())
}

/// `void it_8029C504(HSD_GObj* parent, Vec3* pos, enum_t msid, int kind, f32 angle, f32 speed)`: spawns a shot.
pub unsafe extern "C" fn it_8029C504(parent: *mut HSD_GObj, pos: *mut Vec, msid: enum_t, kind: i32, mut angle: f32, speed: f32) {
    melee_core::ct!("it_8029C504");
    let mut spawn: SpawnItem = core::mem::zeroed();

    Item_NormalizeAngle(&mut angle);
    spawn.kind = kind;
    Item_InitSpawnPositionFromParent(&mut spawn, parent, pos);
    let mut right_facing = true;
    if !((angle as f64) < M_PI / 2.0) && !((angle as f64) > M_PI * 3.0 / 2.0) {
        right_facing = false;
    }
    Item_InitSpawnCommonFields(&mut spawn, parent, if right_facing { 1.0 } else { -1.0 }, true);
    let item_gobj = ext::Item_80268B18(&mut spawn);
    if !item_gobj.is_null() {
        let item = item_of(item_gobj);
        let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut FoxLaserAttr;
        ext::Item_80268E5C(item_gobj, msid, ITEM_ANIM_UPDATE);
        ext::it_80275158(item_gobj, (*attr).lifetime);
        (*item).xDD4_itemVar.foxlaser.scale = 0.0;
        (*item).xDD4_itemVar.foxlaser.angle = angle;
        (*item).xDD4_itemVar.foxlaser.speed = speed;
        (*item).xDD4_itemVar.foxlaser.pos = spawn.pos;
        ext::db_80225DD8(item_gobj, parent);
    }
}

/// `void it_8029C6A4(f32 angle, f32 vel, HSD_GObj* parent, Vec3* vec, int kind)`: Fox's laser (item state 0).
pub unsafe extern "C" fn it_8029C6A4(angle: f32, vel: f32, parent: *mut HSD_GObj, vec: *mut Vec, kind: i32) {
    melee_core::ct!("it_8029C6A4");
    it_8029C504(parent, vec, 0, kind, angle, vel);
}

/// `void it_8029C6CC(f32 angle, f32 vel, HSD_GObj* parent, Vec3* vec, int kind)`: Falco's laser (item state 1).
pub unsafe extern "C" fn it_8029C6CC(angle: f32, vel: f32, parent: *mut HSD_GObj, vec: *mut Vec, kind: i32) {
    melee_core::ct!("it_8029C6CC");
    it_8029C504(parent, vec, 1, kind, angle, vel);
}

/// `bool itFoxlaser_UnkMotion1_Anim(Item_GObj*)`.
pub unsafe extern "C" fn itFoxlaser_UnkMotion1_Anim(item_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFoxlaser_UnkMotion1_Anim");
    let ip = item_of(item_gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut FoxLaserAttr;
    let jobj = jobj_of(item_gobj);
    Item_UpdateRayAnimation(item_gobj, ip, jobj, &raw const (*attr).scale, 11.25)
}

/// `void itFoxlaser_UnkMotion1_Phys(Item_GObj*)`: remembers the position.
pub unsafe extern "C" fn itFoxlaser_UnkMotion1_Phys(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itFoxlaser_UnkMotion1_Phys");
    let item = item_of(item_gobj);
    (*item).xDD4_itemVar.foxlaser.pos = (*item).pos;
}

/// `bool itFoxlaser_UnkMotion1_Coll(Item_GObj*)`: on a map hit, shortens the lifetime to one frame and keeps the position.
pub unsafe extern "C" fn itFoxlaser_UnkMotion1_Coll(item_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFoxlaser_UnkMotion1_Coll");
    let item = item_of(item_gobj);
    let pos = (*item).pos;
    if it_8029C4D4(item_gobj, null_mut()) != 0 {
        ext::it_80275158(item_gobj, 1.0);
        (*item).pos = pos;
    }
    0
}

/// `bool itFoxLaser_Logic94_Clanked(Item_GObj*)`.
pub unsafe extern "C" fn itFoxLaser_Logic94_Clanked(arg0: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFoxLaser_Logic94_Clanked");
    1
}

/// `bool itFoxLaser_Logic94_Reflected(Item_GObj*)`: turns around and restarts the ray.
pub unsafe extern "C" fn itFoxLaser_Logic94_Reflected(item_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFoxLaser_Logic94_Reflected");
    let item = item_of(item_gobj);
    let jobj = jobj_of(item_gobj);
    if (*item).facing_dir != (*item).xC68 {
        (*item).facing_dir = (*item).xC68;
        ji::HSD_JObjSetRotationY(jref(jobj), (M_PI / 2.0 * (*item).facing_dir as f64) as f32, &mut Dirty);
    }
    Item_ResetRayAfterReflection(item, jobj);
    0
}

/// `bool itFoxLaser_Logic94_Absorbed(Item_GObj*)`.
pub unsafe extern "C" fn itFoxLaser_Logic94_Absorbed(arg0: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFoxLaser_Logic94_Absorbed");
    1
}

/// `bool itFoxLaser_Logic94_ShieldBounced(Item_GObj*)`.
pub unsafe extern "C" fn itFoxLaser_Logic94_ShieldBounced(item_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFoxLaser_Logic94_ShieldBounced");
    Item_BounceRayOffShield(item_gobj)
}

/// `bool itFoxLaser_Logic94_HitShield(Item_GObj*)`.
pub unsafe extern "C" fn itFoxLaser_Logic94_HitShield(arg0: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFoxLaser_Logic94_HitShield");
    1
}

/// `void itFoxLaser_Logic94_EvtUnk(Item_GObj*, HSD_GObj*)`: removes the gobj interaction references.
pub unsafe extern "C" fn itFoxLaser_Logic94_EvtUnk(item_gobj: *mut Item_GObj, r#ref: *mut HSD_GObj) {
    melee_core::ct!("itFoxLaser_Logic94_EvtUnk");
    ext::it_8026B894(item_gobj, r#ref);
}
