//! Port of src/melee/it/kinds/itlgunbeam.c: the ray gun's bouncing beam (spawn, motion, wall bounce angles).
//! Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked against the C
//! (whole TU, shipping contraction) by tests/kinds_itlgunbeam.rs.
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
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_802694CC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_802694CC(a0) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn efLib_DestroyAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_DestroyAll(a0) };
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj, a3: *mut f32, a4: *mut f32) -> *mut c_void;
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026D9A0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D9A0(a0) };
    fn it_80272940(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272940(a0) };
    fn it_80273130(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273130(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn lbVector_Angle(a0: *mut Vec, a1: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_Angle(a0, a1) };
    fn lbVector_Normalize(a0: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_Normalize(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

const M_PI: f64 = core::f64::consts::PI;
const M_PI_2: f64 = M_PI / 2.0;
const M_TAU: f64 = 6.283185307179586;

const Collide_FloorMask: i32 = 0x8000 | 0x10000;
const Collide_CeilingMask: i32 = 0x2000 | 0x4000;

/// `ABS` (platform.h): `((x) < 0 ? -(x) : (x))`.
#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `Item_ClampAngle` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_ClampAngle(angle: *mut f32) {
    while (*angle as f64) < -M_PI {
        *angle = (*angle as f64 + M_TAU) as f32;
    }
    while *angle as f64 > M_PI {
        *angle = (*angle as f64 - M_TAU) as f32;
    }
}

/// `Item_ClampAngleReverse` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_ClampAngleReverse(angle: *mut f32) {
    while *angle as f64 > M_PI {
        *angle = (*angle as f64 - M_TAU) as f32;
    }
    while (*angle as f64) < -M_PI {
        *angle = (*angle as f64 + M_TAU) as f32;
    }
}

/// `Item_ClearCmdVars` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_ClearCmdVars(ip: *mut Item) {
    (*ip).xDB8_itcmd_var3 = 0;
    (*ip).xDB4_itcmd_var2 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDAC_itcmd_var0 = 0;
}

/// `Item_ClearFlagsAndEnterState` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_ClearFlagsAndEnterState(gobj: *mut Item_GObj, ip: *mut Item, msid: i32) {
    ext::it_8026B3A8(gobj);
    (*ip).xDC8_word.flags.set_x13(0);
    ext::it_80272940(gobj);
    ext::Item_80268E5C(gobj, msid, 2);
}

/// `Item_InitSpawnPositionFromParent` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitSpawnPositionFromParent(spawn: *mut SpawnItem, parent: *mut HSD_GObj, pos: *mut Vec3) {
    (*spawn).prev_pos = *pos;
    (*spawn).prev_pos.z = 0.0;
    ext::it_8026BB68(parent, &mut (*spawn).pos);
}

/// `ItemStateTable it_803F6630[]`
pub static mut it_803F6630: [ItemStateTable; 1] = [
    st_row(0, pred_b(itLgunbeam_UnkMotion0_Anim), Some(itLgunbeam_UnkMotion0_Phys), pred_b(itLgunbeam_UnkMotion0_Coll)),
];

/// 2993E0: sums the contact normals selected by `flags` into `position1`.
pub unsafe extern "C" fn it_802993E0(gobj: *mut Item_GObj, flags: i32) {
    melee_core::ct!("it_802993E0");
    let item = GET_ITEM(gobj);
    let mut pos: Vec3 = core::mem::zeroed();
    pos.z = 0.0;
    pos.y = 0.0;
    pos.x = 0.0;
    Item_ClampAngle(&mut (*item).xDD4_itemVar.lgunbeam.angle0);
    if flags & (1 << 0) != 0 {
        pos.x += (*item).x378_itemColl.floor.normal.x;
        pos.y += (*item).x378_itemColl.floor.normal.y;
    }
    if flags & (1 << 1) != 0 {
        pos.x += (*item).x378_itemColl.ceiling.normal.x;
        pos.y += (*item).x378_itemColl.ceiling.normal.y;
    }
    if flags & (1 << 2) != 0 {
        pos.x += (*item).x378_itemColl.left_facing_wall.normal.x;
        pos.y += (*item).x378_itemColl.left_facing_wall.normal.y;
    }
    if flags & (1 << 3) != 0 {
        pos.x += (*item).x378_itemColl.right_facing_wall.normal.x;
        pos.y += (*item).x378_itemColl.right_facing_wall.normal.y;
    }
    ext::lbVector_Normalize(&mut pos);
    (*item).xDD4_itemVar.lgunbeam.position1 = pos;
}

/// 299528: turns the beam's heading after a bounce.
pub unsafe extern "C" fn it_80299528(gobj: *mut Item_GObj, arg1: i32) {
    melee_core::ct!("it_80299528");
    let ip = GET_ITEM(gobj);
    if arg1 != 0 {
        let angle1 = ext::atan2f((*ip).xDD4_itemVar.lgunbeam.position1.x, (*ip).xDD4_itemVar.lgunbeam.position1.y);
        let mut angle2 = ext::atan2f((*ip).xDD4_itemVar.lgunbeam.velocity.x, (*ip).xDD4_itemVar.lgunbeam.velocity.y) - angle1;
        Item_ClampAngleReverse(&mut angle2);
        let var_f1: f32;
        if angle2 == 0.0 {
            var_f1 = 0.0;
        } else {
            let temp_f1 = ext::lbVector_Angle(&mut (*ip).xDD4_itemVar.lgunbeam.position1, &mut (*ip).xDD4_itemVar.lgunbeam.velocity);
            let var_f2_2: f32;
            if (abs_c(angle2) as f64) < M_PI_2 {
                var_f2_2 = (0.02f32 as f64 * (abs_c(angle2) as f64 / M_PI)) as f32;
            } else {
                var_f2_2 = (0.5f64 * (abs_c(angle2) as f64 / M_PI)) as f32;
            }
            let var_f0: f32 = if angle2 < 0.0 { -var_f2_2 } else { var_f2_2 };
            var_f1 = temp_f1 * var_f0;
        }
        (*ip).xDD4_itemVar.lgunbeam.angle0 -= var_f1;
        Item_ClampAngle(&mut (*ip).xDD4_itemVar.lgunbeam.angle0);
    }
}

/// 2996D0: spawns a beam at `pos` heading out of `owner_gobj`.
pub unsafe extern "C" fn it_802996D0(owner_gobj: *mut HSD_GObj, pos: *mut Vec3, arg2: u32, facing_dir: f32) -> *mut Item_GObj {
    melee_core::ct!("it_802996D0");
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = It_Kind_L_Gun_Beam;
    Item_InitSpawnPositionFromParent(&mut spawn, owner_gobj, pos);
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = owner_gobj;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = arg2 as _;

    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        let item = GET_ITEM(gobj);
        let item_spec_attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut ItLGunBeamAttr;
        Item_ClearCmdVars(item);
        (*item).xDD4_itemVar.lgunbeam.lifetime = (*item_spec_attr).lifetime;
        ext::it_80275158(gobj, (*item).xDD4_itemVar.lgunbeam.lifetime);
        (*item).xDD4_itemVar.lgunbeam.position0 = *pos;
        {
            let r = ext::HSD_Randf();
            let lo = (*item_spec_attr).x4;
            let diff = (*item_spec_attr).x8 - lo;
            // `diff * r + lo` is one expression: contracted to fmadds (f64 multiply-add, one rounding to f32).
            (*item).xDD4_itemVar.lgunbeam.angle1 = fmadds(diff, r, lo);
        }
        {
            let r = ext::HSD_Randf();
            let lo = (*item_spec_attr).xC;
            let diff = (*item_spec_attr).x10 - lo;
            (*item).xDD4_itemVar.lgunbeam.angle0 = fmadds(diff, r, lo);
        }
        {
            let angle = if (*item).facing_dir == 1.0 {
                (*item).xDD4_itemVar.lgunbeam.angle0
            } else {
                -(*item).xDD4_itemVar.lgunbeam.angle0
            };
            (*item).xDD4_itemVar.lgunbeam.angle0 = angle;
        }
        while ((*item).xDD4_itemVar.lgunbeam.angle0 as f64) < -M_PI {
            (*item).xDD4_itemVar.lgunbeam.angle0 = ((*item).xDD4_itemVar.lgunbeam.angle0 as f64 + M_TAU) as f32;
        }
        while (*item).xDD4_itemVar.lgunbeam.angle0 as f64 > M_PI {
            (*item).xDD4_itemVar.lgunbeam.angle0 = ((*item).xDD4_itemVar.lgunbeam.angle0 as f64 - M_TAU) as f32;
        }
        (*item).xDD4_itemVar.lgunbeam.xE04 = 0;
        (*item).xDCC_flag.set_b3(0);

        it_802998A0(gobj, owner_gobj, ext::HSD_Randi(4));
    }
    gobj
}

/// 2998A0: enters state 0 and spawns the beam's effect.
pub unsafe extern "C" fn it_802998A0(gobj: *mut Item_GObj, fighter_gobj: *mut HSD_GObj, arg2: i32) {
    melee_core::ct!("it_802998A0");
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let mut sp2C: f32 = 1.0;
    Item_ClearFlagsAndEnterState(gobj, ip, 0);
    let pos: Vec3 = (*ip).pos;
    ext::Item_802694CC(gobj);
    itLgunbeam_UnkMotion0_Phys(gobj);
    (*ip).pos.x -= (*ip).x40_vel.x;
    (*ip).pos.y -= (*ip).x40_vel.y;
    ext::it_8026D9A0(gobj);
    (*ip).pos = pos;
    ext::efSync_Spawn(arg2.wrapping_add(0x44E), gobj, jobj, &mut (*ip).facing_dir, &mut sp2C);
    ext::db_80225DD8(gobj, fighter_gobj);
}

pub unsafe extern "C" fn itLgunbeam_UnkMotion0_Anim(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itLgunbeam_UnkMotion0_Anim");
    if ext::it_80273130(gobj) == 1 {
        ext::efLib_DestroyAll(gobj);
        return true;
    }
    false
}

pub unsafe extern "C" fn itLgunbeam_UnkMotion0_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itLgunbeam_UnkMotion0_Phys");
    let ip = GET_ITEM(gobj);

    (*ip).xDD4_itemVar.lgunbeam.position0 = (*ip).pos;

    let vx = (*ip).xDD4_itemVar.lgunbeam.angle1 * sinf((*ip).xDD4_itemVar.lgunbeam.angle0);
    (*ip).x40_vel.x = vx;
    (*ip).xDD4_itemVar.lgunbeam.velocity.x = vx;

    let vy = (*ip).xDD4_itemVar.lgunbeam.angle1 * cosf((*ip).xDD4_itemVar.lgunbeam.angle0);
    (*ip).x40_vel.y = vy;
    (*ip).xDD4_itemVar.lgunbeam.velocity.y = vy;

    (*ip).x40_vel.z = 0.0;
    (*ip).xDD4_itemVar.lgunbeam.velocity.z = 0.0;

    ext::lbVector_Normalize(&mut (*ip).xDD4_itemVar.lgunbeam.velocity);
}

pub unsafe extern "C" fn itLgunbeam_UnkMotion0_Coll(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itLgunbeam_UnkMotion0_Coll");
    let ip = GET_ITEM(gobj);
    Item_ClampAngle(&mut (*ip).xDD4_itemVar.lgunbeam.angle0);
    let mut flags: i32 = 0;
    (*ip).x378_itemColl.ecb_source.anon_4.anon_0_0.up = 3.0;
    (*ip).x378_itemColl.ecb_source.anon_4.anon_0_0.down = 3.0;
    (*ip).x378_itemColl.ecb_source.anon_4.anon_0_0.front = 3.0;
    (*ip).x378_itemColl.ecb_source.anon_4.anon_0_0.back = 3.0;
    ext::it_8026D9A0(gobj);

    if (*ip).x378_itemColl.env_flags & Collide_FloorMask != 0 {
        flags |= 1;
    }
    if (*ip).x378_itemColl.env_flags & Collide_CeilingMask != 0 {
        flags |= 2;
    }
    if (*ip).x378_itemColl.env_flags & 0x3F != 0 {
        flags |= 4;
    }
    if (*ip).x378_itemColl.env_flags & 0xFC0 != 0 {
        flags |= 8;
    }
    let flags2 = flags;
    if flags2 != 0 {
        it_802993E0(gobj, flags2);
        it_80299528(gobj, flags2);
    }
    false
}

pub unsafe extern "C" fn itLGunBeam_Logic39_DmgDealt(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLGunBeam_Logic39_DmgDealt");
    false
}

pub unsafe extern "C" fn itLGunBeam_Logic39_Reflected(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLGunBeam_Logic39_Reflected");
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.lgunbeam.angle0 = ((*ip).xDD4_itemVar.lgunbeam.angle0 as f64 + M_PI) as f32;
    Item_ClampAngle(&mut (*ip).xDD4_itemVar.lgunbeam.angle0);
    (*ip).facing_dir = -(*ip).facing_dir;
    (*ip).x40_vel.x = -(*ip).x40_vel.x;
    (*ip).x40_vel.y = -(*ip).x40_vel.y;
    false
}

pub unsafe extern "C" fn itLGunBeam_Logic39_Clanked(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLGunBeam_Logic39_Clanked");
    true
}

pub unsafe extern "C" fn itLGunBeam_Logic39_Absorbed(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLGunBeam_Logic39_Absorbed");
    false
}

pub unsafe extern "C" fn itLGunBeam_Logic39_ShieldBounced(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLGunBeam_Logic39_ShieldBounced");
    true
}

pub unsafe extern "C" fn itLGunBeam_Logic39_HitShield(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLGunBeam_Logic39_HitShield");
    true
}

pub unsafe extern "C" fn itLGunBeam_Logic39_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("itLGunBeam_Logic39_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}
