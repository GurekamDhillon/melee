//! Port of src/melee/it/kinds/itlgunray.c: Samus-less L-Gun ray item (ray spawn, per-frame motion, reflect/bounce).
//! Callees of other TUs go through `ext`; `bool` is the MSL `int`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_assignments, unused_variables, unused_mut, dead_code, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::gekko::{cosf, sinf};

// BEGIN GENERATED EXT (rust/melee_items/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn atan2f(a0: f32, a1: f32) -> f32;
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn it_8026B3A8(a0: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec);
    fn it_80273130(a0: *mut HSD_GObj) -> i32;
    fn it_80275158(a0: *mut HSD_GObj, a1: f32);
    fn it_8029C4D4(a0: *mut HSD_GObj, a1: *mut CollData) -> i32;
    fn lbVector_Mirror(a0: *mut Vec, a1: *mut Vec);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut dir: f32 = unsafe { core::mem::zeroed() };
    pub static mut ip: *mut Item = unsafe { core::mem::zeroed() };
    pub static mut result: i32 = unsafe { core::mem::zeroed() };
    pub static mut vel_x: f32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

const M_PI: f64 = core::f64::consts::PI;
const M_PI_2: f64 = M_PI / 2.0;
const M_TAU: f64 = 6.283185307179586;

#[inline(always)]
unsafe fn item(gobj: *mut HSD_GObj) -> *mut Item {
    (*gobj).user_data as *mut Item
}

/// `GET_JOBJ`.
#[inline(always)]
unsafe fn jobj_of(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}

/// `ABS`.
#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `HSD_JObjMtxIsDirty` (header inline).
#[inline(always)]
unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    assert!(!jobj.is_null(), "HSD_ASSERT(564, jobj)");
    (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
}

/// `HSD_JObjSetMtxDirty` (macro, and the `(HSD_JObjSetMtxDirty)(..)` function form).
#[inline(always)]
unsafe fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) {
    if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

macro_rules! jobj_setter {
    ($name:ident, $line:expr, |$j:ident, $v:ident : $t:ty| $body:block) => {
        #[inline(always)]
        unsafe fn $name($j: *mut HSD_JObj, $v: $t) {
            assert!(!$j.is_null(), concat!("HSD_ASSERT(", $line, ", jobj)"));
            $body
            if (*$j).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty($j);
            }
        }
    };
}
jobj_setter!(HSD_JObjSetRotationX, 639, |j, x: f32| { assert!((*j).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(640)"); (*j).rotate.x = x; });
jobj_setter!(HSD_JObjSetRotationY, 660, |j, y: f32| { assert!((*j).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(661)"); (*j).rotate.y = y; });
jobj_setter!(HSD_JObjSetRotationZ, 681, |j, z: f32| { assert!((*j).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(682)"); (*j).rotate.z = z; });
jobj_setter!(HSD_JObjSetScaleZ, 806, |j, z: f32| { (*j).scale.z = z; });
jobj_setter!(HSD_JObjSetTranslateY, 947, |j, y: f32| { (*j).translate.y = y; });
jobj_setter!(HSD_JObjSetTranslateZ, 962, |j, z: f32| { (*j).translate.z = z; });
jobj_setter!(HSD_JObjAddRotationX, 1029, |j, x: f32| { (*j).rotate.x += x; });
jobj_setter!(HSD_JObjAddRotationY, 1041, |j, y: f32| { (*j).rotate.y += y; });
jobj_setter!(HSD_JObjAddTranslationY, 1114, |j, y: f32| { (*j).translate.y += y; });
jobj_setter!(HSD_JObjSetRotation, 618, |j, r: *mut Quaternion| { assert!(!r.is_null()); (*j).rotate = *r; });
jobj_setter!(HSD_JObjSetScale, 760, |j, s: *mut Vec3| { assert!(!s.is_null()); (*j).scale = *s; });
jobj_setter!(HSD_JObjSetTranslate, 916, |j, t: *mut Vec3| { assert!(!t.is_null()); (*j).translate = *t; });
use HSD_JObjAddRotationX as HSD_JObjAddRotationXWithMtxDirty;
use HSD_JObjAddTranslationY as HSD_JObjAddTranslationYWithMtxDirty;
use HSD_JObjSetRotation as HSD_JObjSetRotationWithMtxDirty;
use HSD_JObjSetScale as HSD_JObjSetScaleWithMtxDirty;
use HSD_JObjSetTranslate as HSD_JObjSetTranslateWithMtxDirty;

#[inline(always)]
unsafe fn HSD_JObjGetRotationX(jobj: *mut HSD_JObj) -> f32 {
    assert!(!jobj.is_null(), "HSD_ASSERT(715, jobj)");
    (*jobj).rotate.x
}
#[inline(always)]
unsafe fn HSD_JObjGetTranslationY(jobj: *mut HSD_JObj) -> f32 {
    assert!(!jobj.is_null(), "HSD_ASSERT(1006, jobj)");
    (*jobj).translate.y
}
#[inline(always)]
unsafe fn HSD_JObjGetRotation(jobj: *mut HSD_JObj, r: *mut Quaternion) {
    assert!(!jobj.is_null() && !r.is_null());
    *r = (*jobj).rotate;
}
#[inline(always)]
unsafe fn HSD_JObjGetScale(jobj: *mut HSD_JObj, s: *mut Vec3) {
    assert!(!jobj.is_null() && !s.is_null());
    *s = (*jobj).scale;
}

/// `itResetVelocity` (it/inlines.h).
#[inline(always)]
unsafe fn itResetVelocity(ip: *mut Item) {
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.z = 0.0;
}

/// `Item_ClearCmdVars` (it/inlines.h).
#[inline(always)]
unsafe fn Item_ClearCmdVars(ip: *mut Item) {
    (*ip).xDB8_itcmd_var3 = 0;
    (*ip).xDB4_itcmd_var2 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDAC_itcmd_var0 = 0;
}

/// `Item_ApplyFallingPhysics` (it/inlines.h).
#[inline(always)]
unsafe fn Item_ApplyFallingPhysics(gobj: *mut HSD_GObj) {
    let attrs = (*item(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
}

/// `Item_StopAndEnterState` (kinds/inlines.h).
#[inline(always)]
unsafe fn Item_StopAndEnterState(gobj: *mut HSD_GObj, ip: *mut Item, msid: i32) {
    itResetVelocity(ip);
    ext::it_8026B390(gobj);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE as u32);
}

/// `Item_ClearFlagsAndEnterState` (kinds/inlines.h).
#[inline(always)]
unsafe fn Item_ClearFlagsAndEnterState(gobj: *mut HSD_GObj, ip: *mut Item, msid: i32) {
    ext::it_8026B3A8(gobj);
    (*ip).xDC8_word.flags.set_x13(0);
    ext::it_80272940(gobj);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE as u32);
}

/// `Item_ClampAngle` / `Item_ClampAngleReverse` / `Item_NormalizeAngle` (kinds/inlines.h; M_PI, M_TAU are doubles).
#[inline(always)]
unsafe fn Item_ClampAngle(angle: *mut f32) {
    while (*angle as f64) < -M_PI { *angle = (*angle as f64 + M_TAU) as f32; }
    while (*angle as f64) > M_PI { *angle = (*angle as f64 - M_TAU) as f32; }
}
#[inline(always)]
unsafe fn Item_ClampAngleReverse(angle: *mut f32) {
    while (*angle as f64) > M_PI { *angle = (*angle as f64 - M_TAU) as f32; }
    while (*angle as f64) < -M_PI { *angle = (*angle as f64 + M_TAU) as f32; }
}
#[inline(always)]
unsafe fn Item_NormalizeAngle(angle: *mut f32) {
    while *angle < 0.0 { *angle = (*angle as f64 + M_TAU) as f32; }
    while (*angle as f64) > M_TAU { *angle = (*angle as f64 - M_TAU) as f32; }
}

/// `Item_UpdateZakoVelocity` / `Item_ZakoDefeat` / `Item_InitZakoCollision` (kinds/inlines.h).
#[inline(always)]
unsafe fn Item_UpdateZakoVelocity(gobj: *mut HSD_GObj, ip: *mut Item) {
    ext::it_802762BC(ip);
    ext::it_8027BA54(gobj, &raw mut (*ip).x40_vel);
    ext::it_802762BC(ip);
}
#[inline(always)]
unsafe fn Item_ZakoDefeat(gobj: *mut HSD_GObj, ip: *mut Item) {
    ext::it_8027C9D8(ip);
    ext::it_802756D0(gobj);
    ext::it_80275474(gobj);
    ext::it_8027CE44(gobj);
    ext::Camera_RequestQuake(QuakeKind_Small as u32, &raw mut (*ip).pos);
}
#[inline(always)]
unsafe fn Item_InitZakoCollision(gobj: *mut HSD_GObj, ip: *mut Item) {
    (*ip).xD5C = 0;
    (*ip).xDC8_word.flags.set_x15(0);
    ext::it_8027542C(gobj);
    ext::it_80275270(gobj);
    (*ip).xDC8_word.flags.set_x19(1);
}

/// `Item_InitSpawnPositionFromParent` / `Item_InitSpawnPosition` / `Item_InitSpawnCommonFields` (kinds/inlines.h).
#[inline(always)]
unsafe fn Item_InitSpawnPositionFromParent(spawn: *mut SpawnItem, parent: *mut HSD_GObj, pos: *mut Vec3) {
    (*spawn).prev_pos = *pos;
    (*spawn).prev_pos.z = 0.0;
    ext::it_8026BB68(parent, &raw mut (*spawn).pos);
}
#[inline(always)]
unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec3, on_plane: bool) {
    (*spawn).prev_pos = *pos;
    if on_plane {
        (*spawn).prev_pos.z = 0.0;
    }
    (*spawn).pos = (*spawn).prev_pos;
}
#[inline(always)]
unsafe fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing_dir: f32, initial_collision: bool) {
    (*spawn).facing_dir = facing_dir;
    (*spawn).x3C_damage = 0;
    (*spawn).vel.x = 0.0;
    (*spawn).vel.y = 0.0;
    (*spawn).vel.z = 0.0;
    (*spawn).x0_parent_gobj = parent;
    (*spawn).x4_parent_gobj2 = (*spawn).x0_parent_gobj;
    (*spawn).x44_flag.anon_0.set_b0(initial_collision as _);
    (*spawn).x40 = 0;
}

/// `Item_UpdateRayAnimation` / `Item_BounceRayOffShield` / `Item_ResetRayAfterReflection` (kinds/inlines.h).
#[inline(always)]
unsafe fn Item_UpdateRayAnimation(gobj: *mut HSD_GObj, ip: *mut Item, jobj: *mut HSD_JObj, max_scale: *const f32, scale_divisor: f32) -> i32 {
    (*ip).x40_vel.x = (*ip).xDD4_itemVar.ray.speed * cosf((*ip).xDD4_itemVar.ray.angle);
    (*ip).x40_vel.y = (*ip).xDD4_itemVar.ray.speed * sinf((*ip).xDD4_itemVar.ray.angle);
    (*ip).x40_vel.z = 0.0;
    let dir = if (*ip).x40_vel.x > 0.0 { 1.0f32 } else { -1.0f32 };
    (*ip).facing_dir = dir;
    HSD_JObjSetRotationY(jobj, (M_PI_2 * (*ip).facing_dir as f64) as f32);
    let vel_x = if (*ip).facing_dir == 1.0 { -(*ip).x40_vel.x } else { (*ip).x40_vel.x };
    HSD_JObjSetRotationX(jobj, (M_PI + ext::atan2f((*ip).x40_vel.y, vel_x) as f64) as f32);
    (*ip).xDD4_itemVar.ray.scale += abs_c((*ip).xDD4_itemVar.ray.speed) / scale_divisor;
    if (*ip).xDD4_itemVar.ray.scale > *max_scale {
        (*ip).xDD4_itemVar.ray.scale = *max_scale;
    }
    if (*ip).xDD4_itemVar.ray.scale < 1e-5f32 {
        (*ip).xDD4_itemVar.ray.scale = 1e-3;
    }
    HSD_JObjSetScaleZ(jobj, (*ip).xDD4_itemVar.ray.scale);
    ext::it_80273130(gobj)
}
#[inline(always)]
unsafe fn Item_BounceRayOffShield(gobj: *mut HSD_GObj) -> i32 {
    let ip = item(gobj);
    ext::lbVector_Mirror(&raw mut (*ip).x40_vel, &raw mut (*ip).xC58);
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
#[inline(always)]
unsafe fn Item_ResetRayAfterReflection(ip: *mut Item, jobj: *mut HSD_JObj) {
    (*ip).xDD4_itemVar.ray.scale = 1e-3;
    HSD_JObjSetScaleZ(jobj, (*ip).xDD4_itemVar.ray.scale);
    (*ip).xDD4_itemVar.ray.angle = ((*ip).xDD4_itemVar.ray.angle as f64 + M_PI) as f32;
    while (*ip).xDD4_itemVar.ray.angle < 0.0 {
        (*ip).xDD4_itemVar.ray.angle = ((*ip).xDD4_itemVar.ray.angle as f64 + M_TAU) as f32;
    }
    while ((*ip).xDD4_itemVar.ray.angle as f64) > M_TAU {
        (*ip).xDD4_itemVar.ray.angle = ((*ip).xDD4_itemVar.ray.angle as f64 - M_TAU) as f32;
    }
}

pub static mut it_803F64F8: [ItemStateTable; 1] = [ItemStateTable {
    anim_id: 0,
    animated: Some(itLgunray_UnkMotion0_Anim),
    physics_updated: Some(itLgunray_UnkMotion0_Phys),
    collided: Some(itLgunray_UnkMotion0_Coll),
}];

pub unsafe extern "C" fn it_80298168(owner_gobj: *mut HSD_GObj, pos: *mut Vec3, facing_dir: f32) {
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = It_Kind_L_Gun_Ray as _;
    Item_InitSpawnPositionFromParent(&raw mut spawn, owner_gobj, pos);
    Item_InitSpawnCommonFields(&raw mut spawn, owner_gobj, facing_dir, true);
    let gobj = ext::Item_80268B18(&raw mut spawn);
    if !gobj.is_null() {
        let ip = item(gobj);
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

pub unsafe extern "C" fn it_802982AC(gobj: *mut HSD_GObj, owner_gobj: *mut HSD_GObj, article: *mut Article) {
    let da = (*(*item(gobj)).xC4_article_data).x4_specialAttributes as *mut ItLGunRayAttr;
    ext::it_80275158(gobj, (*da).lifetime);
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itLgunray_UnkMotion0_Anim(gobj: *mut HSD_GObj) -> i32 {
    let ip = item(gobj);
    let item_spec_attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut ItLGunRayAttr;
    let jobj = jobj_of(gobj);
    Item_UpdateRayAnimation(gobj, ip, jobj, &raw const (*item_spec_attr).max_scale, 7.0)
}

pub unsafe extern "C" fn itLgunray_UnkMotion0_Phys(gobj: *mut HSD_GObj) {
    let ip = item(gobj);
    (*ip).xDD4_itemVar.lgunray.pos = (*ip).pos;
}

pub unsafe extern "C" fn itLgunray_UnkMotion0_Coll(gobj: *mut HSD_GObj) -> i32 {
    let ip = item(gobj);
    if ext::it_8029C4D4(gobj, &raw mut (*ip).x378_itemColl) != 0 {
        return 1;
    }
    0
}

pub unsafe extern "C" fn itLGunRay_Logic35_DmgDealt(gobj: *mut HSD_GObj) -> i32 { 1 }
pub unsafe extern "C" fn itLGunRay_Logic35_Clanked(gobj: *mut HSD_GObj) -> i32 { 1 }
pub unsafe extern "C" fn itLGunRay_Logic35_HitShield(gobj: *mut HSD_GObj) -> i32 { 1 }
pub unsafe extern "C" fn itLGunRay_Logic35_Absorbed(gobj: *mut HSD_GObj) -> i32 { 1 }

pub unsafe extern "C" fn itLGunRay_Logic35_Reflected(gobj: *mut HSD_GObj) -> i32 {
    let ip = item(gobj);
    let jobj = jobj_of(gobj);
    (*ip).facing_dir = -(*ip).facing_dir;
    HSD_JObjSetRotationY(jobj, (M_PI_2 * (*ip).facing_dir as f64) as f32);
    (*ip).xDD4_itemVar.lgunray.speed *= (*ip).xC70;
    Item_ResetRayAfterReflection(ip, jobj);
    0
}

pub unsafe extern "C" fn itLGunRay_Logic35_ShieldBounced(gobj: *mut HSD_GObj) -> i32 {
    Item_BounceRayOffShield(gobj)
}

pub unsafe extern "C" fn itLGunRay_Logic35_EvtUnk(gobj: *mut HSD_GObj, ref_gobj: *mut HSD_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
