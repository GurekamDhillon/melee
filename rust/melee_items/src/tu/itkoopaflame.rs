//! Native port of src/melee/it/kinds/itkoopaflame.c: Bowser's fire breath (and Kirby's copy of it):
//! spawn with the random speed / angle draw, the per-frame scale / hitbox lifetime, the velocity from
//! the angle, the wall / floor / ceiling bounce direction and angle update, the reflect / shield
//! callbacks and the one-state table `ItemStateTable_KoopaFlame`. Callees in other TUs go through
//! `ext` (hooks; unset panics "TODO"). Checked against the C (the whole TU, shipping contraction) by
//! tests/tu_itkoopaflame.rs.
//!
//! Header inlines compiled into the TU: `Item_ClampAngle` / `Item_ClampAngleReverse` /
//! `Item_InitSpawnPositionFromParent` / `Item_ClearFlagsAndEnterState` (kinds/inlines.h),
//! `Item_ClearCmdVars` (it/inlines.h), `HSD_JObjSetScale` / `HSD_JObjMtxIsDirty` (jobj.h, with their
//! `HSD_ASSERT`s and the `HSD_JObjSetMtxDirty` macro), `GET_ITEM`.
//!
//! Float math: the contracted sites are the two `range * rng + min` draws in `itKoopaFlame_Spawn`
//! (Gekko `fmadds`, see `melee_core::fp::fmadds`). Angle wraps use the f64 constants `M_PI` / `M_TAU`
//! and round to f32 on the store, as in the C; `sinf` / `cosf` are the MSL ports of melee_runtime.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_802694CC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_802694CC(a0) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn efLib_DestroyAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_DestroyAll(a0) };
    fn efSync_Spawn_N(a0: i32, a1: *mut HSD_GObj, a2: &[*mut c_void]) -> *mut c_void;
    fn ftKb_SpecialNKp_800FA83C(a0: *mut HSD_GObj) -> i32;
    fn ftKb_SpecialNKp_800FA878(a0: *mut HSD_GObj) -> i32;
    fn ftKp_SpecialLw_80134DE0(a0: *mut HSD_GObj) -> i32;
    fn ftKp_SpecialLw_80134E1C(a0: *mut HSD_GObj) -> i32;
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026D9A0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D9A0(a0) };
    fn it_802725D4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802725D4(a0) };
    fn it_80272940(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272940(a0) };
    fn it_80273130(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273130(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn lbVector_Angle(a0: *mut Vec, a1: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_Angle(a0, a1) };
    fn lbVector_Mirror(a0: *mut Vec, a1: *mut Vec) => unsafe { melee_runtime::tu::lbvector::lbVector_Mirror(a0, a1) };
    fn lbVector_Normalize(a0: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_Normalize(a0) };
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

const M_PI: f64 = 3.14159265358979323846;
const M_PI_2: f64 = M_PI / 2.0;
const M_TAU: f64 = 6.283185307179586;

pub const itkpf_Floor: i32 = 1;
pub const itkpf_Ceiling: i32 = 2;
pub const itkpf_LeftWall: i32 = 4;
pub const itkpf_RightWall: i32 = 8;

const Collide_LeftWallMask: i32 = 0x3F;
const Collide_RightWallMask: i32 = 0xFC0;
const Collide_CeilingMask: i32 = 0x2000 | 0x4000;
const Collide_FloorMask: i32 = 0x8000 | 0x10000;

/// The TU-local `itKoopaFlame_Attributes` (the article's special attributes).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct itKoopaFlame_Attributes {
    pub x0_lifetime: f32,
    pub x4_hitbox_lifetime: f32,
    pub x8_min_speed: f32,
    pub xC_max_speed: f32,
    pub x10_min_angle: f32,
    pub x14_max_angle: f32,
}

#[inline]
unsafe fn get_item(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `ABS(x)` (macros.h): `x < 0 ? -x : x`.
#[inline]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `HSD_ASSERT(line, cond)`: the failing path is never taken by well-formed state.
#[inline]
fn hsd_assert(cond: bool, what: &str) {
    if !cond {
        panic!("HSD_ASSERT failed: {what}");
    }
}

/// `HSD_JObjMtxIsDirty` (jobj.h).
#[inline]
unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    hsd_assert(!jobj.is_null(), "jobj");
    let mut result = false;
    if (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0 {
        result = true;
    }
    result
}

/// `HSD_JObjSetScale` (jobj.h) with the `HSD_JObjSetMtxDirty` macro.
#[inline]
unsafe fn HSD_JObjSetScale(jobj: *mut HSD_JObj, scale: *mut Vec) {
    hsd_assert(!jobj.is_null(), "jobj");
    hsd_assert(!scale.is_null(), "scale");
    (*jobj).scale = *scale;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 && !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

/// `Item_ClampAngle` (kinds/inlines.h).
#[inline]
unsafe fn Item_ClampAngle(angle: *mut f32) {
    while ((*angle) as f64) < -M_PI {
        *angle = (*angle as f64 + M_TAU) as f32;
    }
    while (*angle) as f64 > M_PI {
        *angle = (*angle as f64 - M_TAU) as f32;
    }
}

/// `Item_ClampAngleReverse` (kinds/inlines.h).
#[inline]
unsafe fn Item_ClampAngleReverse(angle: *mut f32) {
    while (*angle) as f64 > M_PI {
        *angle = (*angle as f64 - M_TAU) as f32;
    }
    while ((*angle) as f64) < -M_PI {
        *angle = (*angle as f64 + M_TAU) as f32;
    }
}

/// `Item_InitSpawnPositionFromParent` (kinds/inlines.h).
#[inline]
unsafe fn Item_InitSpawnPositionFromParent(spawn: *mut SpawnItem, parent: *mut HSD_GObj, pos: *mut Vec) {
    (*spawn).prev_pos = *pos;
    (*spawn).prev_pos.z = 0.0;
    ext::it_8026BB68(parent, &mut (*spawn).pos);
}

/// `Item_ClearCmdVars` (it/inlines.h).
#[inline]
unsafe fn Item_ClearCmdVars(ip: *mut Item) {
    (*ip).xDB8_itcmd_var3 = 0;
    (*ip).xDB4_itcmd_var2 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDAC_itcmd_var0 = 0;
}

/// `Item_ClearFlagsAndEnterState` (kinds/inlines.h).
#[inline]
unsafe fn Item_ClearFlagsAndEnterState(gobj: *mut Item_GObj, ip: *mut Item, msid: i32) {
    ext::it_8026B3A8(gobj);
    (*ip).xDC8_word.flags.set_x13(0);
    ext::it_80272940(gobj);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE as _);
}

pub static mut ItemStateTable_KoopaFlame: [ItemStateTable; 1] = [ItemStateTable {
    anim_id: 0,
    animated: Some(itKoopaFlame_UnkMotion0_Anim),
    physics_updated: Some(itKoopaFlame_UnkMotion0_Phys),
    collided: Some(itKoopaFlame_UnkMotion0_Coll),
}];

pub unsafe extern "C" fn itKoopaFlame_Update_Direction(gobj: *mut Item_GObj, flags: i32) {
    melee_core::ct!("itKoopaFlame_Update_Direction");
    let it = get_item(gobj);
    let mut vec: Vec = (*it).xDD4_itemVar.koopaflame.xC_direction;
    Item_ClampAngle(&mut (*it).xDD4_itemVar.koopaflame.x24_angle);
    if flags & itkpf_Floor != 0 {
        vec.x += (*it).x378_itemColl.floor.normal.x;
        vec.y += (*it).x378_itemColl.floor.normal.y;
    }
    if flags & itkpf_Ceiling != 0 {
        vec.x += (*it).x378_itemColl.ceiling.normal.x;
        vec.y += (*it).x378_itemColl.ceiling.normal.y;
    }
    if flags & itkpf_LeftWall != 0 {
        vec.x += (*it).x378_itemColl.left_facing_wall.normal.x;
        vec.y += (*it).x378_itemColl.left_facing_wall.normal.y;
    }
    if flags & itkpf_RightWall != 0 {
        vec.x += (*it).x378_itemColl.right_facing_wall.normal.x;
        vec.y += (*it).x378_itemColl.right_facing_wall.normal.y;
    }
    ext::lbVector_Normalize(&mut vec);
    (*it).xDD4_itemVar.koopaflame.xC_direction = vec;
}

pub unsafe extern "C" fn itKoopaFlame_Update_Angle(gobj: *mut Item_GObj, flags: i32) {
    melee_core::ct!("itKoopaFlame_Update_Angle");
    let it = get_item(gobj);
    if flags != 0 {
        let a = ext::atan2f((*it).xDD4_itemVar.koopaflame.x18_vel.x, (*it).xDD4_itemVar.koopaflame.x18_vel.y);
        let b = ext::atan2f((*it).xDD4_itemVar.koopaflame.xC_direction.x, (*it).xDD4_itemVar.koopaflame.xC_direction.y);
        let mut f31 = a - b;
        let mut f1: f32;
        Item_ClampAngleReverse(&mut f31);
        if f31 == 0.0 {
            f1 = 0.0;
        } else {
            f1 = ext::lbVector_Angle(&mut (*it).xDD4_itemVar.koopaflame.xC_direction, &mut (*it).xDD4_itemVar.koopaflame.x18_vel);
            let f2: f32 = if (abs_c(f31) as f64) < M_PI_2 {
                (0.02f32 as f64 * (abs_c(f31) as f64 / M_PI)) as f32
            } else {
                (0.5f32 as f64 * (abs_c(f31) as f64 / M_PI)) as f32
            };
            let f0: f32 = if f31 < 0.0 { -f2 } else { f2 };
            f1 *= f0;
        }
        (*it).xDD4_itemVar.koopaflame.x24_angle -= f1;
        Item_ClampAngle(&mut (*it).xDD4_itemVar.koopaflame.x24_angle);
    }
}

pub unsafe extern "C" fn itKoopaFlame_Spawn(
    parent: *mut Fighter_GObj,
    pos: *mut Vec,
    facing_dir: f32,
    unk: u32,
    gfx: i32,
    base_speed: i32,
    scale: i32,
    kind: i32,
) -> *mut Item_GObj {
    melee_core::ct!("itKoopaFlame_Spawn");
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = kind as _;
    Item_InitSpawnPositionFromParent(&mut spawn, parent, pos);
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.x = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.z = 0.0;
    spawn.x0_parent_gobj = parent;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = unk as _;
    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        let it = (*gobj).user_data as *mut Item;
        let attrs = (*(*it).xC4_article_data).x4_specialAttributes as *mut itKoopaFlame_Attributes;
        Item_ClearCmdVars(it);
        (*it).xDD4_itemVar.koopaflame.x2C_lifetime = (*attrs).x0_lifetime;
        ext::it_80275158(gobj, (*it).xDD4_itemVar.koopaflame.x2C_lifetime);
        (*it).xDD4_itemVar.koopaflame.x0_pos = *pos;
        (*it).xDD4_itemVar.koopaflame.x34_base_scale = 0.0;
        match (*it).kind as i32 {
            k if k == It_Kind_Koopa_Flame as i32 => {
                (*it).xDD4_itemVar.koopaflame.x38_base_speed = base_speed as f32 / ext::ftKp_SpecialLw_80134DE0((*it).owner) as f32;
                (*it).xDD4_itemVar.koopaflame.x3C_scale = scale as f32 / ext::ftKp_SpecialLw_80134E1C((*it).owner) as f32;
            }
            k if k == It_Kind_Kirby_KoopaFlame as i32 => {
                (*it).xDD4_itemVar.koopaflame.x38_base_speed = base_speed as f32 / ext::ftKb_SpecialNKp_800FA83C((*it).owner) as f32;
                (*it).xDD4_itemVar.koopaflame.x3C_scale = scale as f32 / ext::ftKb_SpecialNKp_800FA878((*it).owner) as f32;
            }
            _ => {}
        }
        let mut rng = ext::HSD_Randf();
        let min = (*attrs).x8_min_speed;
        let mut range = (*attrs).xC_max_speed - min;
        (*it).xDD4_itemVar.koopaflame.x28_speed = (*it).xDD4_itemVar.koopaflame.x38_base_speed * fmadds(range, rng, min);
        rng = ext::HSD_Randf();
        range = (*attrs).x14_max_angle - (*attrs).x10_min_angle;
        (*it).xDD4_itemVar.koopaflame.x24_angle = fmadds(range, rng, (*attrs).x10_min_angle);
        (*it).xDD4_itemVar.koopaflame.x24_angle = if (*it).facing_dir == 1.0 {
            (*it).xDD4_itemVar.koopaflame.x24_angle
        } else {
            -(*it).xDD4_itemVar.koopaflame.x24_angle
        };
        Item_ClampAngle(&mut (*it).xDD4_itemVar.koopaflame.x24_angle);
        (*it).xDD4_itemVar.koopaflame.x40_frame_counter = 0;
        (*it).xDD4_itemVar.koopaflame.x30 = 0;
        (*it).xDD4_itemVar.koopaflame.x44_spawned = 0;
        (*it).xDD4_itemVar.koopaflame.x48_gfx = gfx;
        (*it).xDD4_itemVar.koopaflame.xC_direction.x = (*it).facing_dir;
        (*it).xDD4_itemVar.koopaflame.xC_direction.z = 0.0;
        (*it).xDD4_itemVar.koopaflame.xC_direction.y = 0.0;
        (*it).xDCC_flag.set_b3(0);
        itKoopaFlame_Setup(gobj, parent, gfx);
    }
    gobj
}

pub unsafe extern "C" fn itKoopaFlame_Setup(gobj_i: *mut Item_GObj, gobj_f: *mut Fighter_GObj, _unk: i32) {
    melee_core::ct!("itKoopaFlame_Setup");
    let it = get_item(gobj_i);
    Item_ClearFlagsAndEnterState(gobj_i, it, 0);
    let vec = (*it).pos;
    ext::Item_802694CC(gobj_i);
    itKoopaFlame_UnkMotion0_Phys(gobj_i);
    (*it).pos.x -= (*it).x40_vel.x;
    (*it).pos.y -= (*it).x40_vel.y;
    ext::it_8026D9A0(gobj_i);
    (*it).pos = vec;
    ext::db_80225DD8(gobj_i, gobj_f);
}

pub unsafe extern "C" fn itKoopaFlame_UnkMotion0_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKoopaFlame_UnkMotion0_Anim");
    let it = get_item(gobj);
    let attrs = (*(*it).xC4_article_data).x4_specialAttributes as *mut itKoopaFlame_Attributes;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut vec: Vec = core::mem::zeroed();
    if (*it).x5D4_hitboxes[0].hit.state != HitCapsule_Disabled {
        if (*it).xDD4_itemVar.koopaflame.x34_base_scale == 0.0 {
            (*it).xDD4_itemVar.koopaflame.x34_base_scale = (*it).x5D4_hitboxes[0].hit.scale;
        }
        (*it).x5D4_hitboxes[0].hit.scale = (*it).xDD4_itemVar.koopaflame.x34_base_scale * (*it).xDD4_itemVar.koopaflame.x3C_scale;
    }
    vec.x = (*it).xDD4_itemVar.koopaflame.x3C_scale;
    vec.y = vec.x;
    vec.z = vec.x;
    HSD_JObjSetScale(jobj, &mut vec);
    (*it).xDD4_itemVar.koopaflame.x40_frame_counter = (*it).xDD4_itemVar.koopaflame.x40_frame_counter.wrapping_add(1);
    if (*it).xDD4_itemVar.koopaflame.x40_frame_counter as f32 > (*attrs).x4_hitbox_lifetime {
        ext::it_802725D4(gobj);
    }
    if (*it).xDD4_itemVar.koopaflame.x44_spawned == 0 {
        (*it).xDD4_itemVar.koopaflame.x44_spawned = 1;
        match (*it).kind as i32 {
            k if k == It_Kind_Koopa_Flame as i32 => {
                ext::efSync_Spawn_N(1243i32.wrapping_add((*it).xDD4_itemVar.koopaflame.x48_gfx), gobj, &[jobj as *mut c_void]);
            }
            k if k == It_Kind_Kirby_KoopaFlame as i32 => {
                ext::efSync_Spawn_N(1189i32.wrapping_add((*it).xDD4_itemVar.koopaflame.x48_gfx), gobj, &[jobj as *mut c_void]);
            }
            _ => {}
        }
    }
    ext::it_80273130(gobj)
}

pub unsafe extern "C" fn itKoopaFlame_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itKoopaFlame_UnkMotion0_Phys");
    let it = get_item(gobj);
    (*it).xDD4_itemVar.koopaflame.x0_pos = (*it).pos;
    let vx = (*it).xDD4_itemVar.koopaflame.x28_speed * melee_runtime::gekko::sinf((*it).xDD4_itemVar.koopaflame.x24_angle);
    (*it).x40_vel.x = vx;
    (*it).xDD4_itemVar.koopaflame.x18_vel.x = vx;
    let vy = (*it).xDD4_itemVar.koopaflame.x28_speed * melee_runtime::gekko::cosf((*it).xDD4_itemVar.koopaflame.x24_angle);
    (*it).x40_vel.y = vy;
    (*it).xDD4_itemVar.koopaflame.x18_vel.y = vy;
    (*it).x40_vel.z = 0.0;
    (*it).xDD4_itemVar.koopaflame.x18_vel.z = 0.0;
    ext::lbVector_Normalize(&mut (*it).xDD4_itemVar.koopaflame.x18_vel);
}

pub unsafe extern "C" fn itKoopaFlame_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKoopaFlame_UnkMotion0_Coll");
    let it = get_item(gobj);
    Item_ClampAngle(&mut (*it).xDD4_itemVar.koopaflame.x24_angle);
    (*it).x378_itemColl.ecb_source.anon_4.anon_0_0.up = 3.0;
    (*it).x378_itemColl.ecb_source.anon_4.anon_0_0.down = 3.0;
    (*it).x378_itemColl.ecb_source.anon_4.anon_0_0.front = 3.0;
    (*it).x378_itemColl.ecb_source.anon_4.anon_0_0.back = 3.0;
    let mut flags: i32 = 0;
    ext::it_8026D9A0(gobj);
    if (*it).x378_itemColl.env_flags & Collide_FloorMask != 0 {
        flags |= itkpf_Floor;
    }
    if (*it).x378_itemColl.env_flags & Collide_CeilingMask != 0 {
        flags |= itkpf_Ceiling;
    }
    if (*it).x378_itemColl.env_flags & Collide_LeftWallMask != 0 {
        flags |= itkpf_LeftWall;
    }
    if (*it).x378_itemColl.env_flags & Collide_RightWallMask != 0 {
        flags |= itkpf_RightWall;
    }
    let flags2 = flags;
    if flags2 != 0 {
        itKoopaFlame_Update_Direction(gobj, flags2);
        itKoopaFlame_Update_Angle(gobj, flags2);
    }
    0
}

pub unsafe extern "C" fn itKoopaFlame_Logic111_DmgDealt(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKoopaFlame_Logic111_DmgDealt");
    0
}

pub unsafe extern "C" fn itKoopaFlame_Logic111_Reflected(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKoopaFlame_Logic111_Reflected");
    let it = get_item(gobj);
    (*it).xDD4_itemVar.koopaflame.x24_angle = ((*it).xDD4_itemVar.koopaflame.x24_angle as f64 + M_PI) as f32;
    Item_ClampAngle(&mut (*it).xDD4_itemVar.koopaflame.x24_angle);
    (*it).facing_dir = -(*it).facing_dir;
    (*it).x40_vel.x = -(*it).x40_vel.x;
    (*it).x40_vel.y = -(*it).x40_vel.y;
    0
}

pub unsafe extern "C" fn itKoopaFlame_Logic111_Clanked(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKoopaFlame_Logic111_Clanked");
    0
}

pub unsafe extern "C" fn itKoopaFlame_Logic111_Absorbed(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKoopaFlame_Logic111_Absorbed");
    ext::efLib_DestroyAll(gobj);
    1
}

pub unsafe extern "C" fn itKoopaFlame_Logic111_ShieldBounced(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKoopaFlame_Logic111_ShieldBounced");
    let it = get_item(gobj);
    ext::lbVector_Mirror(&mut (*it).x40_vel, &mut (*it).xC58);
    (*it).x40_vel.z = 0.0;
    (*it).xDD4_itemVar.koopaflame.x24_angle = ext::atan2f((*it).x40_vel.y, (*it).x40_vel.x);
    Item_ClampAngle(&mut (*it).xDD4_itemVar.koopaflame.x24_angle);
    0
}

pub unsafe extern "C" fn itKoopaFlame_Logic111_HitShield(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKoopaFlame_Logic111_HitShield");
    0
}

pub unsafe extern "C" fn itKoopaFlame_Logic111_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itKoopaFlame_Logic111_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}
