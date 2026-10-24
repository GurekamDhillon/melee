//! Port of src/melee/it/itzako.c: the "zako" (enemy-item) spawn and motion helpers: the spawn with a
//! jobj-relative position, the knockback velocity, the orientation that follows the ground normal, the
//! Game&Watch and the Destroyed/Defeated hooks. Callees of other TUs go through `ext` (unset hooks panic
//! "TODO"). Checked against the C by the state oracle `tests/tu_itzako.rs` (gen/mtu).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_mut, unused_assignments, unused_variables, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::{fmadds, frsqrte};
use melee_runtime::gekko::{cosf, sinf};
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
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Camera_GetTransformPosition(a0: *mut Vec);
    fn Ground_801C5840() -> i32 => unsafe { melee_world::tu::ground::Ground_801C5840() };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268B5C(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B5C(a0) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn ScriptGame_EnemyDefeated(a0: *mut HSD_GObj) -> i32;
    fn ScriptGame_EnemyDestroyed(a0: *mut HSD_GObj) -> i32;
    fn ftLib_800865C0(a0: *mut HSD_GObj) -> f32 => unsafe { crate::tu::ftlib::ftLib_800865C0(a0) };
    fn ftLib_800870BC(a0: *mut HSD_GObj, a1: *mut i32) -> i32;
    fn ftLib_8008770C(a0: *mut HSD_GObj, a1: *mut c_void);
    fn ftLib_80087744(a0: *mut HSD_GObj, a1: *mut c_void);
    fn ftLib_GetMotionId(a0: *mut HSD_GObj) -> i32;
    fn gm_IsCurrently1PMode() -> i32;
    fn grLib_801C9E40() -> i32 => unsafe { melee_world::tu::grlib::grLib_801C9E40() };
    fn grLib_801C9E50(a0: i16) => unsafe { melee_world::tu::grlib::grLib_801C9E50(a0) };
    fn grZakoGenerator_801CAC14(a0: *mut HSD_GObj) => unsafe { melee_world::tu::grzakogenerator::grZakoGenerator_801CAC14(a0) };
    fn grZakoGenerator_801CACB8(a0: *mut HSD_GObj) => unsafe { melee_world::tu::grzakogenerator::grZakoGenerator_801CACB8(a0) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B684(a0: *mut Vec) -> f32 => unsafe { crate::tu::it_26B1::it_8026B684(a0 as _) };
    fn it_8026BB88(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB88(a0, a1 as _) };
    fn it_8026C220(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026C220(a0, a1) };
    fn it_8026DAA8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DAA8(a0) };
    fn it_80274594(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274594(a0) };
    fn it_80274740(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274740(a0) };
    fn it_80275228(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275228(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275240(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275240(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275270(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275270(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275414(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275414(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027542C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8027542c(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275444(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275444(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275474(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275474(&mut *((*a0).user_data as *mut Item)) };
    fn it_802754D4(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802754d4(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_8027781C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itmaplib::it_8027781C(a0) as _ };
    fn it_80278574(a0: *mut HSD_GObj, a1: *mut _GXColor) => unsafe { crate::tu::itmaterial::it_80278574(a0, a1) };
    fn it_802F2094(a0: *mut HSD_GObj, a1: *mut Vec, a2: i32, a3: i32) -> *mut HSD_GObj => unsafe { crate::tu::itcoin::it_802F2094(a0, a1 as _, a2, a3) };
    fn lbVector_Angle(a0: *mut Vec, a1: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_Angle(a0, a1) };
    fn lbVector_AngleXY(a0: *mut Vec, a1: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_AngleXY(a0, a1) };
    fn lbVector_CrossprodNormalized(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> *mut Vec => unsafe { melee_runtime::tu::lbvector::lbVector_CrossprodNormalized(a0, a1, a2) };
    fn lbVector_Diff(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> *mut Vec => unsafe { melee_runtime::tu::lbvector::lbVector_Diff(a0, a1, a2) };
    fn lbVector_EulerAnglesFromONB(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec) -> *mut Vec => unsafe { melee_runtime::tu::lbvector::lbVector_EulerAnglesFromONB(a0, a1, a2, a3) };
    fn lbVector_Normalize(a0: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_Normalize(a0) };
    fn lbVector_Rotate(a0: *mut Vec, a1: i32, a2: f32) => unsafe { melee_runtime::tu::lbvector::lbVector_Rotate(a0, a1, a2) };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
    fn pl_8004049C(a0: i32, a1: i32);
    fn tyDisplay_8031C354(a0: i32, a1: *mut i32, a2: i32, a3: i32) -> i32 => unsafe { crate::tu::tydisplay::tyDisplay_8031C354(a0, a1, a2, a3) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D28: *mut ItemCommonData = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D40: *mut it_804D6D40_t = unsafe { core::mem::zeroed() };
    pub static mut p_ftCommonData: *mut ftCommonData = unsafe { core::mem::zeroed() };
    pub static mut result: i32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// `HSD_JObjMtxIsDirty` (header inline).
#[inline(always)]
unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    assert!(!jobj.is_null(), "HSD_ASSERT(564, jobj)");
    (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
}

/// `HSD_JObjSetMtxDirty` (macro).
#[inline(always)]
unsafe fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) {
    if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

#[inline(always)]
unsafe fn HSD_JObjSetRotationX(jobj: *mut HSD_JObj, v: f32) {
    assert!(!jobj.is_null(), "HSD_ASSERT(649, jobj)");
    assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(650, !(jobj->flags & JOBJ_USE_QUATERNION))");
    (*jobj).rotate.x = v;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

#[inline(always)]
unsafe fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, v: f32) {
    assert!(!jobj.is_null(), "HSD_ASSERT(665, jobj)");
    assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(666, !(jobj->flags & JOBJ_USE_QUATERNION))");
    (*jobj).rotate.y = v;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

#[inline(always)]
unsafe fn HSD_JObjSetRotationZ(jobj: *mut HSD_JObj, v: f32) {
    assert!(!jobj.is_null(), "HSD_ASSERT(681, jobj)");
    assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(682, !(jobj->flags & JOBJ_USE_QUATERNION))");
    (*jobj).rotate.z = v;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

#[inline(always)]
unsafe fn HSD_JObjGetRotationX(jobj: *mut HSD_JObj) -> f32 {
    assert!(!jobj.is_null(), "HSD_ASSERT(715, jobj)");
    (*jobj).rotate.x
}

#[inline(always)]
unsafe fn HSD_JObjGetRotationY(jobj: *mut HSD_JObj) -> f32 {
    assert!(!jobj.is_null(), "HSD_ASSERT(730, jobj)");
    (*jobj).rotate.y
}

#[inline(always)]
unsafe fn HSD_JObjGetRotationZ(jobj: *mut HSD_JObj) -> f32 {
    assert!(!jobj.is_null(), "HSD_ASSERT(745, jobj)");
    (*jobj).rotate.z
}

/// `it_803B85A8`: the six reference unit vectors (rest directions) used by the orientation code.
pub static it_803B85A8: [Vec3; 6] = [
    Vec3 { x: 1.0, y: 0.0, z: 0.0 },
    Vec3 { x: 0.0, y: 0.0, z: 1.0 },
    Vec3 { x: 0.0, y: 1.0, z: 0.0 },
    Vec3 { x: 0.0, y: 1.0, z: 0.0 },
    Vec3 { x: 0.0, y: 1.0, z: 0.0 },
    Vec3 { x: 0.0, y: 1.0, z: 0.0 },
];

/// `Collide_*` masks (src/melee/mp/forward.h).
const Collide_FloorMask: i32 = 0x8000 | 0x10000;
const Collide_CeilingMask: i32 = 0x2000 | 0x4000;

/// `M_PI_2` (a double macro in platform.h).
const M_PI_2: f64 = 1.5707963267948966;

#[inline(always)]
unsafe fn item_of(gobj: *mut HSD_GObj) -> *mut Item {
    (*gobj).user_data as *mut Item
}

#[inline(always)]
unsafe fn jobj_of(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}

/// `it_8027B5B0`: spawns a zako item at `pos` (or the jobj's origin when `pos` is NULL).
pub unsafe extern "C" fn it_8027B5B0(kind: ItemKind, pos: *mut Vec3, jobj: *mut HSD_JObj, vel: *mut Vec3, use_init: i32) -> *mut HSD_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    let mut sp20: Vec3 = core::mem::zeroed();
    let gobj: *mut HSD_GObj;
    let ip: *mut Item;

    spawn.kind = kind;
    if !pos.is_null() {
        spawn.prev_pos = *pos;
    } else if !jobj.is_null() {
        ext::lb_8000B1CC(jobj, null_mut(), &raw mut sp20);
        spawn.prev_pos = sp20;
    } else {
        // The C spins here forever.
        loop {
            core::hint::spin_loop();
        }
    }
    spawn.pos = spawn.prev_pos;
    spawn.facing_dir = ext::it_8026B684(&raw mut spawn.prev_pos);
    spawn.x3C_damage = 0;
    spawn.x44_flag.anon_0.set_b0(1);
    if !vel.is_null() {
        spawn.vel = *vel;
    } else {
        spawn.vel.x = 0.0;
        spawn.vel.y = 0.0;
        spawn.vel.z = 0.0;
    }
    spawn.x0_parent_gobj = null_mut();
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x40 = 0;
    if use_init == 1 {
        gobj = ext::Item_80268B18(&raw mut spawn);
    } else {
        gobj = ext::Item_80268B5C(&raw mut spawn);
    }
    if !gobj.is_null() {
        ip = item_of(gobj);
        (*ip).xDD4_itemVar.zako.jobj = jobj;
        (*ip).xDD4_itemVar.zako.idx = -1;
        (*ip).xDD4_itemVar.zako.x14.z = 0;
        (*ip).x378_itemColl.x34_flags.set_b1234(3);
    }
    gobj
}

/// `it_8027B730`: resets a zako item's motion state.
pub unsafe extern "C" fn it_8027B730(item_gobj: *mut HSD_GObj) {
    let item = (*item_gobj).user_data as *mut Item;
    (*item).xDD4_itemVar.zako.jobj = null_mut();
    (*item).xDD4_itemVar.zako.idx = -1;
    (*item).xDD4_itemVar.zako.x8.z = 0.0;
    (*item).xDD4_itemVar.zako.x8.x = 0.0;
    (*item).xDD4_itemVar.zako.x8.y = 1.0;
    (*item).xDD4_itemVar.zako.x14.x = 0;
    (*item).xDD4_itemVar.zako.x14.y = 0;
    (*item).xDD4_itemVar.zako.x14.z = 0;
    ext::it_80275228(item_gobj);
    ext::it_80275240(item_gobj);
}

/// `it_8027B798_CalcAngle` (static inline): the knockback launch angle, in radians.
unsafe fn it_8027B798_CalcAngle(ground_or_air: GroundOrAir, angle: i32, common_data: *mut ftCommonData, knockback: f32) -> f32 {
    if angle != 0x169 {
        0.017453292f32 * (angle as f32)
    } else if ground_or_air == GA_Air {
        (*common_data).x144_radians
    } else if knockback < (*common_data).x14C {
        0.0
    } else {
        // (x148 * q) + 1.0 fuses (product on the left).
        let q = ((knockback - (*common_data).x14C) / ((*common_data).x150 - (*common_data).x14C));
        let mut temp_angle = 0.017453292f32 * fmadds((*common_data).x148, q, 1.0);
        if temp_angle > 0.017453292f32 * (*common_data).x148 {
            temp_angle = 0.017453292f32 * (*common_data).x148;
        }
        temp_angle
    }
}

/// `it_8027B798`: the knockback velocity for a zako item; true when it leaves along the floor.
pub unsafe extern "C" fn it_8027B798(gobj: *mut HSD_GObj, out_vel: *mut Vec3) -> bool {
    let ip = item_of(gobj);
    let mut result = false;
    let knockback = (*ip).xCC8_knockback;
    let mag = knockback * (*ext::p_ftCommonData).x100;
    let angle_rad = it_8027B798_CalcAngle((*ip).ground_or_air, (*ip).xCAC_angle, ext::p_ftCommonData, knockback);
    let dx = mag * cosf(angle_rad);
    let dy = mag * sinf(angle_rad);
    (*ip).facing_dir = (*ip).xCCC_incDamageDirection;
    if (*ip).ground_or_air == GA_Air {
        (*out_vel).x = -dx * (*ip).facing_dir;
        (*out_vel).y = dy;
        (*out_vel).z = 0.0;
    } else {
        let normal = &raw mut (*ip).x378_itemColl.floor.normal;
        let mut v = Vec3 { x: -dx * (*ip).facing_dir, y: dy, z: 0.0 };
        if ext::lbVector_Angle(normal, &raw mut v) < 1.5707964f32 {
            result = true;
            (*out_vel).x = v.x;
            (*out_vel).y = v.y;
            (*out_vel).z = 0.0;
        } else {
            let x = v.x;
            (*out_vel).x = (*normal).y * x;
            (*out_vel).y = -(*normal).x * x;
            (*out_vel).z = 0.0;
        }
    }
    result
}

/// `it_8027B964`: sets the zako item's velocity (a knockback launch, or the reset-on-hit path when `chk`).
pub unsafe extern "C" fn it_8027B964(item_gobj: *mut HSD_GObj, chk: i32) {
    let mut sp14: Vec3;
    let item = item_of(item_gobj);
    if chk != 0 {
        ext::it_8027781C(item_gobj);
    } else {
        it_8027B798(item_gobj, &raw mut (*item).x40_vel);
        if (*item).x40_vel.x == 0.0 && (*item).x40_vel.y == 0.0 {
            (*item).x40_vel.x = 0.1f32 * -(*item).xCCC_incDamageDirection;
            (*item).x40_vel.y = 0.1;
        }
        if (*item).x40_vel.y == 0.0 {
            (*item).x40_vel.y = 0.1;
        }
    }
    sp14 = (*item).x40_vel;
    ext::lbVector_Normalize(&raw mut sp14);
    let scale = (*ext::it_804D6D40).x4;
    sp14.x *= scale;
    sp14.y *= scale;
    sp14.z *= scale;
    (*item).x40_vel = sp14;
}

/// `it_8027782C`-style helper of the C file: aims the item velocity at the camera with a random spread.
/// (The C returns an int without a return statement; the value is not used.)
pub unsafe extern "C" fn it_8027BA54(item_gobj: *mut HSD_GObj, arg1: *mut Vec3) -> i32 {
    let item = item_of(item_gobj);
    let mut sp20: Vec3 = core::mem::zeroed();
    let mut sp14: Vec3 = core::mem::zeroed();
    ext::Camera_GetTransformPosition(&raw mut sp20);
    // 10.0f * (ext::HSD_Randf() - 0.5) is a double product, added to a float in double precision (one rounding).
    sp20.x = (10.0f64).mul_add(ext::HSD_Randf() as f64 - 0.5, sp20.x as f64) as f32;
    sp20.y = fmadds(30.0, ext::HSD_Randf(), sp20.y);
    ext::lbVector_Diff(&raw mut sp20, &raw mut (*item).pos, &raw mut sp14);
    ext::lbVector_Normalize(&raw mut sp14);
    sp14.x *= 10.0;
    sp14.y *= 10.0;
    sp14.z *= 10.0;
    *arg1 = sp14;
    0
}

/// `it_8027BB1C`: rotates `arg0` by the Euler angles `arg1` (x, then y, then z).
pub unsafe extern "C" fn it_8027BB1C(arg0: *mut Vec3, arg1: *mut Vec3) {
    let mut cos: f32;
    let mut sin: f32;
    let mut t1: f32;
    let mut t2: f32;
    {
        cos = cosf((*arg1).x);
        sin = sinf((*arg1).x);
        t1 = (*arg0).y;
        t2 = (*arg0).z;
        (*arg0).y = fmadds(t1, cos, -(t2 * sin));
        (*arg0).z = fmadds(t1, sin, t2 * cos);
    }
    {
        cos = cosf((*arg1).y);
        sin = sinf((*arg1).y);
        t1 = (*arg0).x;
        t2 = (*arg0).z;
        (*arg0).x = fmadds(t1, cos, t2 * sin);
        (*arg0).z = fmadds(t2, cos, -(t1 * sin));
    }
    {
        cos = cosf((*arg1).z);
        sin = sinf((*arg1).z);
        t1 = fmadds((*arg0).x, cos, -((*arg0).y * sin));
        t2 = fmadds((*arg0).x, sin, (*arg0).y * cos);
        (*arg0).x = t1;
        (*arg0).y = t2;
    }
}

/// `it_8027BBF4`: turns the item's jobj so its local up follows the floor (`arg_chk` false) or the
/// ceiling (`arg_chk` true) normal, easing toward it over `arg9` frames.
pub unsafe extern "C" fn it_8027BBF4(item_gobj: *mut HSD_GObj, arg_chk: i32, arg8: f64, arg9: f32) {
    let mut temp_f1: f32;
    let mut temp_f30: f32;
    let mut var_f29: f32;
    let mut var_f28: f32;
    let vecs = &it_803B85A8;
    let mut sp74: Vec3 = core::mem::zeroed();
    let mut sp68: Vec3;
    let mut sp5C: Vec3;
    let mut sp50: Vec3;
    let mut sp44: Vec3;
    let mut sp38: Vec3 = core::mem::zeroed();

    let item = item_of(item_gobj);
    let item_jobj = jobj_of(item_gobj);
    let coll = &raw mut (*item).x378_itemColl;
    sp68 = vecs[2];
    sp5C = vecs[3];

    if arg_chk == 0 {
        if (*coll).env_flags & Collide_FloorMask != 0 {
            sp68 = (*coll).floor.normal;
        } else {
            return;
        }
    } else if (*coll).env_flags & Collide_CeilingMask != 0 {
        sp68 = (*coll).ceiling.normal;
    } else {
        return;
    }

    sp74.x = HSD_JObjGetRotationX(item_jobj);
    sp74.y = HSD_JObjGetRotationY(item_jobj);
    sp74.z = HSD_JObjGetRotationZ(item_jobj);

    it_8027BB1C(&raw mut sp5C, &raw mut sp74);
    if ext::lbVector_AngleXY(&raw mut (*item).xDD4_itemVar.zako.x8, &raw mut sp68) > 0.00017453292f32 {
        (*item).xDD4_itemVar.zako.x8 = sp68;
        (*item).xDD4_itemVar.zako.x14.x = arg9 as i32;
    }
    temp_f1 = ext::lbVector_AngleXY(&raw mut sp5C, &raw mut (*item).xDD4_itemVar.zako.x8);
    if temp_f1 < 0.00017453292f32 || (*item).xDD4_itemVar.zako.x14.x < 2 {
        var_f29 = sp68.x;
        var_f28 = sp68.y;
    } else {
        var_f29 = temp_f1 / ((*item).xDD4_itemVar.zako.x14.x as f32);
        (*item).xDD4_itemVar.zako.x14.x -= 1;
        if fmadds(sp5C.x, sp68.y, -(sp5C.y * sp68.x)) < 0.0 {
            var_f29 = -var_f29;
        }
        temp_f30 = cosf(var_f29);
        temp_f1 = sinf(var_f29);
        var_f29 = fmadds(sp5C.x, temp_f30, -(sp5C.y * temp_f1));
        var_f28 = fmadds(sp5C.x, temp_f1, sp5C.y * temp_f30);
    }

    sp44 = vecs[0];
    sp50 = vecs[1];
    ext::lbVector_Rotate(&raw mut sp50, 2, (M_PI_2 * arg8) as f32);
    ext::lbVector_Rotate(&raw mut sp44, 2, (M_PI_2 * arg8) as f32);

    sp38.x = var_f29;
    sp38.y = var_f28;
    sp38.z = 0.0;
    ext::lbVector_CrossprodNormalized(&raw mut sp38, &raw mut sp50, &raw mut sp44);
    ext::lbVector_CrossprodNormalized(&raw mut sp44, &raw mut sp38, &raw mut sp50);
    ext::lbVector_EulerAnglesFromONB(&raw mut sp74, &raw mut sp50, &raw mut sp44, &raw mut sp38);

    HSD_JObjSetRotationX(item_jobj, sp74.x);
    HSD_JObjSetRotationY(item_jobj, sp74.y);
    HSD_JObjSetRotationZ(item_jobj, sp74.z);
}

/// `it_8027C0A8`: `it_8027BBF4` on the floor normal.
pub unsafe extern "C" fn it_8027C0A8(item_gobj: *mut HSD_GObj, arg4: f32, arg5: f32) {
    it_8027BBF4(item_gobj, 0, arg4 as f64, arg5);
}

/// `it_8027C0CC`: `it_8027BBF4` on the ceiling normal.
pub unsafe extern "C" fn it_8027C0CC(item_gobj: *mut HSD_GObj, arg4: f32, arg5: f32) {
    it_8027BBF4(item_gobj, 1, arg4 as f64, arg5);
}

/// `it_8027C0F0`: orients the item to `arg1` (or the reference floor direction) while on the floor.
pub unsafe extern "C" fn it_8027C0F0(item_gobj: *mut HSD_GObj, arg1: *mut Vec3, arg8: f64, arg9: f32) {
    let vecs = &it_803B85A8;
    let item = item_of(item_gobj);
    let item_jobj = jobj_of(item_gobj);
    let mut temp_f1: f32;
    let mut temp_f30: f32;
    let mut var_f29: f32;
    let mut var_f28: f32;
    let mut sp74: Vec3 = core::mem::zeroed();
    let mut sp68: Vec3;
    let mut sp5C: Vec3;
    let mut sp50: Vec3;
    let mut sp44: Vec3;
    let mut sp3C: Vec3 = core::mem::zeroed();

    sp68 = vecs[4];
    sp5C = vecs[5];

    if (*item).x378_itemColl.env_flags & Collide_FloorMask != 0 {
        if !arg1.is_null() {
            sp68 = *arg1;
        }
        sp74.x = HSD_JObjGetRotationX(item_jobj);
        sp74.y = HSD_JObjGetRotationY(item_jobj);
        sp74.z = HSD_JObjGetRotationZ(item_jobj);
        it_8027BB1C(&raw mut sp5C, &raw mut sp74);

        if ext::lbVector_AngleXY(&raw mut (*item).xDD4_itemVar.zako.x8, &raw mut sp68) > 0.00017453292f32 {
            (*item).xDD4_itemVar.zako.x8 = sp68;
            (*item).xDD4_itemVar.zako.x14.x = arg9 as i32;
        }
        temp_f1 = ext::lbVector_AngleXY(&raw mut sp5C, &raw mut (*item).xDD4_itemVar.zako.x8);
        if temp_f1 < 0.00017453292f32 || (*item).xDD4_itemVar.zako.x14.x < 2 {
            var_f29 = sp68.x;
            var_f28 = sp68.y;
        } else {
            var_f29 = temp_f1 / ((*item).xDD4_itemVar.zako.x14.x as f32);
            (*item).xDD4_itemVar.zako.x14.x -= 1;
            if fmadds(sp5C.x, sp68.y, -(sp5C.y * sp68.x)) < 0.0 {
                var_f29 = -var_f29;
            }
            temp_f30 = cosf(var_f29);
            temp_f1 = sinf(var_f29);
            var_f29 = fmadds(sp5C.x, temp_f30, -(sp5C.y * temp_f1));
            var_f28 = fmadds(sp5C.x, temp_f1, sp5C.y * temp_f30);
        }

        sp44 = vecs[0];
        sp50 = vecs[1];
        ext::lbVector_Rotate(&raw mut sp50, 2, (M_PI_2 * arg8) as f32);
        ext::lbVector_Rotate(&raw mut sp44, 2, (M_PI_2 * arg8) as f32);
        sp3C.x = var_f29;
        sp3C.y = var_f28;
        sp3C.z = 0.0;
        ext::lbVector_CrossprodNormalized(&raw mut sp3C, &raw mut sp50, &raw mut sp44);
        ext::lbVector_CrossprodNormalized(&raw mut sp44, &raw mut sp3C, &raw mut sp50);
        ext::lbVector_EulerAnglesFromONB(&raw mut sp74, &raw mut sp50, &raw mut sp44, &raw mut sp3C);

        HSD_JObjSetRotationX(item_jobj, sp74.x);
        HSD_JObjSetRotationY(item_jobj, sp74.y);
        HSD_JObjSetRotationZ(item_jobj, sp74.z);
    }
}

/// `it_8027C56C`: resets the jobj's rotation to face `y_rot` and clears the zako's up vector.
pub unsafe extern "C" fn it_8027C56C(item_gobj: *mut HSD_GObj, y_rot: f32) {
    let item_jobj = jobj_of(item_gobj);
    let item = item_of(item_gobj);
    HSD_JObjSetRotationX(item_jobj, 0.0);
    HSD_JObjSetRotationY(item_jobj, (M_PI_2 * y_rot as f64) as f32);
    HSD_JObjSetRotationZ(item_jobj, 0.0);

    (*item).xDD4_itemVar.zako.x8.z = 0.0;
    (*item).xDD4_itemVar.zako.x8.x = 0.0;
    (*item).xDD4_itemVar.zako.x8.y = 1.0;
    (*item).xDD4_itemVar.zako.x14.x = 0;
}

/// `it_8027C794`: placeholder predicate (always false).
pub unsafe extern "C" fn it_8027C794(gobj: *mut HSD_GObj) -> bool {
    let _ = gobj;
    false
}

/// `it_8027C79C`: counts the zako's timer while it is in a hit-stun state; destroys it when the timer
/// passes the table limit.
pub unsafe extern "C" fn it_8027C79C(item_gobj: *mut HSD_GObj) -> bool {
    if ext::it_8026DAA8(item_gobj) & 0xF != 0 {
        let item = item_of(item_gobj);
        it_8027C9D8(item);
        it_8027B964(item_gobj, 1);
        (*item).xDD4_itemVar.zako.x14.y += 1;
        if (*item).xDD4_itemVar.zako.x14.y > (*ext::it_804D6D40).x0 {
            it_8027CC88(item_gobj);
            return true;
        }
    }
    false
}

/// `it_8027C824`: runs `arg_func` (when given) once the zako's state is 0xF-masked active; the defeat
/// path runs otherwise.
pub unsafe extern "C" fn it_8027C824(item_gobj: *mut HSD_GObj, arg_func: Option<unsafe extern "C" fn(*mut HSD_GObj) -> i32>) -> i32 {
    if ext::it_8026DAA8(item_gobj) & 0xF != 0 {
        it_8027CC88(item_gobj);
        if let Some(f) = arg_func {
            let arg_func_ret_val = f(item_gobj);
            if arg_func_ret_val != 0 {
                it_8027CE44(item_gobj);
            }
            return arg_func_ret_val;
        }
        it_8027CE44(item_gobj);
        return 1;
    }
    0
}

/// `it_2725_Logic9_Destroyed`: the enemy's Destroyed hook (scripted enemies have no generator slot).
pub unsafe extern "C" fn it_2725_Logic9_Destroyed(item_gobj: *mut HSD_GObj) {
    if ext::ScriptGame_EnemyDestroyed(item_gobj) != 0 {
        return;
    }
    ext::grZakoGenerator_801CAC14(item_gobj);
}

/// `product_xyz` (static inline): a.x*b.x + a.y*b.y + a.z*b.z, with the fused products of the C build.
#[inline(always)]
fn product_xyz(a: &Vec3, b: &Vec3) -> f32 {
    // ((x*x + y*y) + z*z): the inner add fuses its left multiply, the outer add fuses z*z
    fmadds(a.z, b.z, fmadds(a.x, b.x, a.y * b.y))
}

/// `itzako_sqrtf` (static inline): three Newton steps on `__frsqrte`, as in the C build.
#[inline(always)]
fn itzako_sqrtf(x: f32) -> f32 {
    if x > 0.0 {
        let mut guess = frsqrte(x as f64);
        guess = 0.5 * guess * ((-(guess * guess)).mul_add(x as f64, 3.0));
        guess = 0.5 * guess * ((-(guess * guess)).mul_add(x as f64, 3.0));
        guess = 0.5 * guess * ((-(guess * guess)).mul_add(x as f64, 3.0));
        let y = (x as f64 * guess) as f32;
        return y;
    }
    x
}

/// `return_sqrt_value3` (static inline): the length of `v`.
#[inline(always)]
fn return_sqrt_value3(v: &Vec3) -> f32 {
    itzako_sqrtf(product_xyz(v, v))
}

/// `it_8027C8D0`: rotates `arg0` (in the XY plane) by the signed angle `arg8`, keeping its length.
pub unsafe extern "C" fn it_8027C8D0(arg0: *mut Vec3, arg1: *mut Vec3, arg8: f32) {
    let dir: f32;
    let var_f6: f32;
    let vec_mag = return_sqrt_value3(&*arg0);
    let sp1c = *arg1;
    if arg8 > 0.0 {
        dir = -1.0;
        var_f6 = 6.8074177e-9;
    } else if arg8 < 0.0 {
        dir = 1.0;
        var_f6 = 6.8074177e-9;
    } else {
        (*arg0).z = 0.0;
        (*arg0).y = 0.0;
        (*arg0).x = 0.0;
        return;
    }
    (*arg0).x = vec_mag * fmadds(sp1c.x, var_f6, -(sp1c.y * dir));
    (*arg0).y = vec_mag * fmadds(sp1c.x, dir, sp1c.y * var_f6);
    (*arg0).z = 0.0;
}

/// `it_8027C9D8`: picks a new sound effect (0x131..0x134, never the same one twice) and plays it.
pub unsafe extern "C" fn it_8027C9D8(item: *mut Item) {
    let spc: [i32; 4] = [0x131, 0x132, 0x133, 0x134];
    let mut var_r3 = spc[ext::HSD_Randi(4) as usize];
    while var_r3 == (*item).xDD4_itemVar.zako.x14.z {
        var_r3 = spc[ext::HSD_Randi(4) as usize];
    }
    (*item).xDD4_itemVar.zako.x14.z = var_r3;
    ext::Item_8026AE84(item, (*item).xDD4_itemVar.zako.x14.z, 127, 64);
}

/// `it_8027CA7C`: true for the two motion ids 0x14D and 0x14E.
pub unsafe extern "C" fn it_8027CA7C(gobj: *mut HSD_GObj) -> bool {
    let mut chk = false;
    if ext::ftLib_GetMotionId(gobj) == 0x14E {
        chk = true;
    }
    if ext::ftLib_GetMotionId(gobj) == 0x14D {
        chk = true;
    }
    chk
}

/// `it_8027CAD8`: clears the zako's velocity and detaches it from its owner.
pub unsafe extern "C" fn it_8027CAD8(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    (*item).x40_vel.y = 0.0;
    (*item).x40_vel.x = 0.0;
    ext::it_8027542C(item_gobj);
    ext::it_80275444(item_gobj);
    ext::it_80274740(item_gobj);
    (*item).owner = null_mut();
    ext::it_80275270(item_gobj);
}

/// `it_8027CB3C`: attaches the zako to its owner (position and facing).
pub unsafe extern "C" fn it_8027CB3C(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    ext::it_8026C220(item_gobj, (*item).owner);
    (*item).facing_dir = ext::ftLib_800865C0((*item).owner);
    ext::it_80275414(item_gobj);
    ext::it_80275474(item_gobj);
    ext::it_802762BC(item);
    ext::it_8026B3A8(item_gobj);
}

/// `it_8027CBA4`: follows the owner's facing without re-attaching.
pub unsafe extern "C" fn it_8027CBA4(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    (*item).facing_dir = ext::ftLib_800865C0((*item).owner);
    ext::it_802762BC(item);
    ext::it_802754D4(item_gobj);
    ext::it_8026B3A8(item_gobj);
}

/// `it_8027CBFC`: the per-kind float the zako uses (from `it_804D6D40`), 0 for other kinds.
pub unsafe extern "C" fn it_8027CBFC(item_gobj: *mut HSD_GObj) -> f32 {
    let mut ret_val: i32 = 0;
    match (*item_of(item_gobj)).xCC4 {
        6 | 7 => ret_val = (*ext::it_804D6D40).xC as i32,
        9 => ret_val = (*ext::it_804D6D40).x10 as i32,
        _ => {}
    }
    ret_val as f32
}

/// `it_8027CC88`: once per the random counter, spawns a coin (kind chosen from the stage table) at the
/// zako's position while the 1P mode is on.
pub unsafe extern "C" fn it_8027CC88(item_gobj_arg: *mut HSD_GObj) -> *mut HSD_GObj {
    let mut temp_r3: i32;
    let mut temp_r3_2: i32;
    let mut var_r30: i32;
    let mut item_gobj_var: *mut HSD_GObj = null_mut();
    let mut var_item: *mut Item;
    let mut sp44: Vec3 = core::mem::zeroed();
    let mut sp1c: [i32; 10] = [0; 10];

    if gm_IsCurrently1PMode() && ext::HSD_Randi((*ext::it_804D6D40).x14 as i32) == 0 {
        if ext::grLib_801C9E40() == 0 {
            var_r30 = ext::Ground_801C5840();
            if var_r30 != -1 {
                temp_r3 = ext::tyDisplay_8031C354(var_r30, sp1c.as_mut_ptr(), 0xA, 0x63);
                temp_r3_2 = temp_r3 + ext::tyDisplay_8031C354(var_r30, sp1c.as_mut_ptr().add(temp_r3 as usize), 0xA, 2);
                if temp_r3_2 != 0 {
                    var_r30 = sp1c[ext::HSD_Randi(temp_r3_2) as usize];
                }
                ext::it_8026BB88(item_gobj_arg, &raw mut sp44);
                item_gobj_var = ext::it_802F2094(null_mut(), &raw mut sp44, var_r30, 0);
                if !item_gobj_var.is_null() {
                    var_item = item_of(item_gobj_var);
                    (*var_item).x40_vel.x = (2.0f32 * (ext::HSD_Randf() - 0.5)) * (*ext::it_804D6D28).x54_float;
                    (*var_item).x40_vel.y = (*(*var_item).xCC_item_attr).x18;
                    (*var_item).x40_vel.z = 0.0;
                    ext::grLib_801C9E50(1);
                }
            }
        } else if (ext::grLib_801C9E40() as f32) >= (*ext::it_804D6D40).x18 {
            ext::grLib_801C9E50(0);
        } else {
            ext::grLib_801C9E50((ext::grLib_801C9E40() + 1) as i16);
        }
    }
    item_gobj_var
}

/// `gm_IsCurrently1PMode` returns bool in the C header.
#[inline(always)]
pub unsafe fn gm_IsCurrently1PMode() -> bool {
    ext::gm_IsCurrently1PMode() != 0
}

/// `it_8027CE18`: plays the defeat sound of the item's source player.
pub unsafe extern "C" fn it_8027CE18(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    ext::pl_8004049C((*item).xCB0_source_ply as i32, (*item).kind);
}

/// `it_8027CE44`: the defeat transition. Scripted enemies are handled by the script game; the stock path
/// hands the zako to the grZakoGenerator.
pub unsafe extern "C" fn it_8027CE44(item_gobj: *mut HSD_GObj) {
    let scripted = ext::ScriptGame_EnemyDefeated(item_gobj);
    if scripted != 0 {
        if scripted == 1 {
            it_8027CE18(item_gobj);
        }
        return;
    }
    ext::grZakoGenerator_801CACB8(item_gobj);
}

/// `it_8027CE64`: the Game&Watch variant: marks the item's flag, copies the fighter's color overlay and
/// stores the attribute address.
pub unsafe extern "C" fn it_8027CE64(item_gobj: *mut HSD_GObj, fighter_gobj: *mut HSD_GObj, arg_attr_address: *mut c_void) {
    let item = item_of(item_gobj);
    let mut sp18: GXColor = core::mem::zeroed();
    (*item).xDCF_flag.anon_0.set_b3(1);
    ext::ftLib_8008770C(fighter_gobj, &raw mut sp18 as *mut c_void);
    ext::it_80278574(item_gobj, &raw mut sp18);
    ext::ftLib_80087744(fighter_gobj, &raw mut (*item).xBC8 as *mut c_void);
    (*item).x5C8 = ext::ftLib_800870BC((*item).owner, &raw mut (*item).xBC4 as *mut i32) as u8;
    ext::it_80274594(item_gobj);
    (*item).xDD4_itemVar.gamewatch.attr = arg_attr_address;
}
