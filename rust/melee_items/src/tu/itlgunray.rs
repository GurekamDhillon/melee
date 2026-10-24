//! Port of src/melee/it/kinds/itlgunray.c: the ray gun's projectile (spawn, per-frame ray animation, reflect/bounce).
//! Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked against the C
//! (whole TU, shipping contraction) by tests/kinds_itlgunray.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_80273130(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273130(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_8029C4D4(a0: *mut HSD_GObj, a1: *mut CollData) -> i32 => unsafe { crate::tu::itfoxlaser::it_8029C4D4(a0 as _, a1) };
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

crate::kinds_jobj_inl!();

const M_PI: f64 = core::f64::consts::PI;
const M_PI_2: f64 = M_PI / 2.0;
const M_TAU: f64 = 6.283185307179586;

/// `ABS` (platform.h): `((x) < 0 ? -(x) : (x))`.
#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
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

/// `Item_InitSpawnPositionFromParent` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitSpawnPositionFromParent(spawn: *mut SpawnItem, parent: *mut HSD_GObj, pos: *mut Vec3) {
    (*spawn).prev_pos = *pos;
    (*spawn).prev_pos.z = 0.0;
    ext::it_8026BB68(parent, &mut (*spawn).pos);
}

/// `Item_UpdateRayAnimation` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_UpdateRayAnimation(gobj: *mut Item_GObj, ip: *mut Item, jobj: *mut HSD_JObj, max_scale: *const f32, scale_divisor: f32) -> i32 {
    let ray = &mut (*ip).xDD4_itemVar.ray;
    (*ip).x40_vel.x = ray.speed * cosf(ray.angle);
    (*ip).x40_vel.y = ray.speed * sinf(ray.angle);
    (*ip).x40_vel.z = 0.0;
    let dir = if (*ip).x40_vel.x > 0.0 { 1.0f32 } else { -1.0f32 };
    (*ip).facing_dir = dir;
    HSD_JObjSetRotationY(jobj, (M_PI_2 * (*ip).facing_dir as f64) as f32);
    let vel_x = if (*ip).facing_dir == 1.0 { -(*ip).x40_vel.x } else { (*ip).x40_vel.x };
    HSD_JObjSetRotationX(jobj, (M_PI + ext::atan2f((*ip).x40_vel.y, vel_x) as f64) as f32);
    ray.scale += abs_c(ray.speed) / scale_divisor;
    if ray.scale > *max_scale {
        ray.scale = *max_scale;
    }
    if ray.scale < 1e-5f32 {
        ray.scale = 1e-3f32;
    }
    HSD_JObjSetScaleZ(jobj, ray.scale);
    ext::it_80273130(gobj)
}

/// `Item_BounceRayOffShield` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_BounceRayOffShield(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    ext::lbVector_Mirror(&mut (*ip).x40_vel, &mut (*ip).xC58);
    (*ip).xDD4_itemVar.ray.scale = 1e-3f32;
    (*ip).xDD4_itemVar.ray.angle = ext::atan2f((*ip).x40_vel.y, (*ip).x40_vel.x);
    while ((*ip).xDD4_itemVar.ray.angle as f64) < 0.0 {
        (*ip).xDD4_itemVar.ray.angle = ((*ip).xDD4_itemVar.ray.angle as f64 + M_TAU) as f32;
    }
    while (*ip).xDD4_itemVar.ray.angle as f64 > M_TAU {
        (*ip).xDD4_itemVar.ray.angle = ((*ip).xDD4_itemVar.ray.angle as f64 - M_TAU) as f32;
    }
    false
}

/// `Item_ResetRayAfterReflection` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_ResetRayAfterReflection(ip: *mut Item, jobj: *mut HSD_JObj) {
    (*ip).xDD4_itemVar.ray.scale = 1e-3f32;
    HSD_JObjSetScaleZ(jobj, (*ip).xDD4_itemVar.ray.scale);
    (*ip).xDD4_itemVar.ray.angle = ((*ip).xDD4_itemVar.ray.angle as f64 + M_PI) as f32;
    while ((*ip).xDD4_itemVar.ray.angle as f64) < 0.0 {
        (*ip).xDD4_itemVar.ray.angle = ((*ip).xDD4_itemVar.ray.angle as f64 + M_TAU) as f32;
    }
    while (*ip).xDD4_itemVar.ray.angle as f64 > M_TAU {
        (*ip).xDD4_itemVar.ray.angle = ((*ip).xDD4_itemVar.ray.angle as f64 - M_TAU) as f32;
    }
}

/// `ItemStateTable it_803F64F8[]`
pub static mut it_803F64F8: [ItemStateTable; 1] = [
    st_row(0, Some(itLgunray_UnkMotion0_Anim), Some(itLgunray_UnkMotion0_Phys), pred_b(itLgunray_UnkMotion0_Coll)),
];

/// 298168: fires a ray from `owner_gobj`.
pub unsafe extern "C" fn it_80298168(owner_gobj: *mut HSD_GObj, pos: *mut Vec3, facing_dir: f32) {
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = It_Kind_L_Gun_Ray;
    Item_InitSpawnPositionFromParent(&mut spawn, owner_gobj, pos);
    Item_InitSpawnCommonFields(&mut spawn, owner_gobj, facing_dir, true);
    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        let article = (*ip).xC4_article_data;
        let item_spec_attr = (*article).x4_specialAttributes as *mut ItLGunRayAttr;
        it_802982AC(gobj, owner_gobj, article);
        (*ip).xDD4_itemVar.lgunray.scale = 0.0;
        (*ip).xDD4_itemVar.lgunray.angle = if (*ip).facing_dir == 1.0 { 0.0 } else { M_PI as f32 };
        (*ip).xDD4_itemVar.lgunray.speed = (*item_spec_attr).speed;
        (*ip).xDD4_itemVar.lgunray.pos = spawn.pos;
        (*ip).x40_vel.x = (*item_spec_attr).speed * (*ip).facing_dir;
        ext::db_80225DD8(gobj, owner_gobj);
    }
}

/// 2982AC
pub unsafe extern "C" fn it_802982AC(gobj: *mut Item_GObj, owner_gobj: *mut Item_GObj, article: *mut Article) {
    let da = (*(*GET_ITEM(gobj)).xC4_article_data).x4_specialAttributes as *mut ItLGunRayAttr;
    ext::it_80275158(gobj, (*da).lifetime);
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 0, 2);
}

/// Returns the raw `int` of `it_80273130` (`bool` is the MSL `int`).
pub unsafe extern "C" fn itLgunray_UnkMotion0_Anim(gobj: *mut Item_GObj) -> i32 {
    let ip = GET_ITEM(gobj);
    let item_spec_attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut ItLGunRayAttr;
    let jobj = GET_JOBJ(gobj);
    Item_UpdateRayAnimation(gobj, ip, jobj, &(*item_spec_attr).max_scale, 7.0)
}

pub unsafe extern "C" fn itLgunray_UnkMotion0_Phys(gobj: *mut HSD_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.lgunray.pos = (*ip).pos;
}

pub unsafe extern "C" fn itLgunray_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if ext::it_8029C4D4(gobj, &mut (*ip).x378_itemColl) != 0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn itLGunRay_Logic35_DmgDealt(_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itLGunRay_Logic35_Clanked(_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itLGunRay_Logic35_HitShield(_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itLGunRay_Logic35_Absorbed(_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itLGunRay_Logic35_Reflected(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    (*ip).facing_dir = -(*ip).facing_dir;
    HSD_JObjSetRotationY(jobj, (M_PI_2 * (*ip).facing_dir as f64) as f32);
    (*ip).xDD4_itemVar.lgunray.speed *= (*ip).xC70;
    Item_ResetRayAfterReflection(ip, jobj);
    false
}

pub unsafe extern "C" fn itLGunRay_Logic35_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    Item_BounceRayOffShield(gobj)
}

pub unsafe extern "C" fn itLGunRay_Logic35_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
