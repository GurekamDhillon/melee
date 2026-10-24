//! Native port of src/melee/it/kinds/itlipstickspore.c: the Lipstick spore particle item (two spawn paths from
//! the fighter / the lipstick, set-up per variant, deceleration, the logic-37 callbacks). Callees in other TUs
//! go through `ext` (hooks; unset panics "TODO"). Checked against the C (the whole TU, shipping contraction)
//! by tests/tu_itlipstickspore.rs. Float math: products and compares only, no `a*b+c` site to fuse
//! (`ABS(vel / x10)` is a division). `efSync_Spawn` is `ext::efSync_Spawn_N(gfx, gobj, &[args])`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;
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
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn efSync_Spawn_N(a0: i32, a1: *mut HSD_GObj, a2: &[*mut c_void]) -> *mut c_void;
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026DFB0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DFB0(a0) };
    fn it_80272460(a0: *mut HitCapsule, a1: u32, a2: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_80272460(a0, a1, a2) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80273130(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273130(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80274740(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274740(a0) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
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

crate::inl_it_core!();
crate::inl_it_spawn_from_parent!();
crate::inl_it_jobj!();

#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `HSD_JObjSetRotationZ` (jobj.h).
#[inline]
unsafe fn HSD_JObjSetRotationZ(jobj: *mut HSD_JObj, z: f32) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
    (*jobj).rotate.z = z;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

pub static mut it_803F6680: [ItemStateTable; 2] = [
    ItemStateTable { anim_id: 0, animated: Some(itLipstickspore_UnkMotion0_Anim), physics_updated: Some(itLipstickspore_UnkMotion0_Phys), collided: Some(itLipstickspore_UnkMotion0_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itLipstickspore_UnkMotion1_Anim), physics_updated: Some(itLipstickspore_UnkMotion1_Phys), collided: Some(itLipstickspore_UnkMotion1_Coll) },
];

pub unsafe fn it_8029A114(gobj: *mut Fighter_GObj, pos: *mut Vec3, mut facing_dir: f32, arg4: i32) {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = It_Kind_LipStick_Spore;
    Item_InitSpawnPositionFromParent(&mut spawn, gobj, pos);
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.x = 0.0;
    spawn.vel.y = spawn.vel.x;
    spawn.vel.z = spawn.vel.y;
    spawn.x0_parent_gobj = gobj;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;
    let item_gobj = ext::Item_80268B18(&mut spawn);
    if !item_gobj.is_null() {
        let ip = (*item_gobj).user_data as *mut Item;
        (*ip).xDD4_itemVar.lipstickspore.xDD4 = arg4;
        it_8029A31C(item_gobj, gobj, ip);
        ext::db_80225DD8(item_gobj, gobj);
        ext::efSync_Spawn_N(0x440, item_gobj, &[pos as *mut c_void, &raw mut facing_dir as *mut c_void]);
    }
}

pub unsafe fn it_8029A218(owner: *mut HSD_GObj, pos: *mut Vec3, mut facing_dir: f32, arg4: i32) {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = It_Kind_LipStick_Spore;
    Item_InitSpawnPositionFromParent(&mut spawn, owner, pos);
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = owner;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;
    let item_gobj = ext::Item_80268B18(&mut spawn);
    if !item_gobj.is_null() {
        let ip = GET_ITEM(item_gobj);
        (*ip).xDD4_itemVar.lipstickspore.xDD4 = arg4;
        it_8029A498(item_gobj, owner);
        ext::db_80225DD8(item_gobj, owner);
        ext::efSync_Spawn_N(0x441, item_gobj, &[pos as *mut c_void, &raw mut facing_dir as *mut c_void]);
    }
}

pub unsafe fn it_8029A31C(gobj: *mut Item_GObj, _owner: *mut Fighter_GObj, _unused: *mut Item) {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itLipstickSporeAttributes;

    ext::it_8026B3A8(gobj);
    ext::it_80274740(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
    let hit = &raw mut (*ip).x5D4_hitboxes[0].hit;

    match (*ip).xDD4_itemVar.lipstickspore.xDD4 {
        0 => {
            ext::it_80275158(gobj, (*attr).x10);
            (*ip).x40_vel.x = (*ip).facing_dir * ((*attr).x0 * (*attr).x8);
            (*ip).xDD4_itemVar.lipstickspore.xDD8 = abs_c((*ip).x40_vel.x / (*attr).x10);
            ext::it_80272460(hit, (*attr).x18 as u32, gobj);
        }
        1 => {
            ext::it_80275158(gobj, (*attr).xC);
            (*ip).x40_vel.x = (*ip).facing_dir * ((*attr).x0 * (*attr).x4);
            (*ip).xDD4_itemVar.lipstickspore.xDD8 = abs_c((*ip).x40_vel.x / (*attr).xC);
            ext::it_80272460(hit, (*attr).x14 as u32, gobj);
        }
        2 => {
            ext::it_80275158(gobj, (*attr).xC);
            (*ip).x40_vel.x = 0.0;
            (*ip).xDD4_itemVar.lipstickspore.xDD8 = 0.0;
            ext::it_80272460(hit, (*attr).x14 as u32, gobj);
        }
        _ => {}
    }
    (*ip).xDCE_flag.anon_0.set_b7(0);
}

pub unsafe fn it_8029A498(gobj: *mut Item_GObj, _owner: *mut Fighter_GObj) {
    let ip = GET_ITEM(gobj);

    ext::it_8026B3A8(gobj);
    ext::it_80274740(gobj);
    ext::it_80275158(gobj, 5.0);
    ext::Item_80268E5C(gobj, 1, 2);

    (*ip).xDCE_flag.anon_0.set_b7(0);
}

pub unsafe extern "C" fn itLipstickspore_UnkMotion0_Anim(gobj: *mut Item_GObj) -> i32 {
    ext::it_80273130(gobj)
}

pub unsafe extern "C" fn itLipstickspore_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);

    if (*ip).x40_vel.x == 0.0 {
        return;
    }

    if (*ip).xDD4_itemVar.lipstickspore.xDD8 == 0.0 {
        return;
    }

    if (*ip).x40_vel.x > 0.0 {
        (*ip).x40_vel.x -= (*ip).xDD4_itemVar.lipstickspore.xDD8;
    } else {
        (*ip).x40_vel.x += (*ip).xDD4_itemVar.lipstickspore.xDD8;
    }

    let mut abs_vel = (*ip).x40_vel.x;
    if abs_vel < 0.0 {
        abs_vel = -abs_vel;
    }
    if abs_vel < (*ip).xDD4_itemVar.lipstickspore.xDD8 {
        (*ip).x40_vel.x = 0.0;
    }
}

pub unsafe extern "C" fn itLipstickspore_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    if ext::it_8026DFB0(gobj) != 0 {
        return 1;
    }
    0
}

pub unsafe extern "C" fn itLipstickspore_UnkMotion1_Anim(gobj: *mut Item_GObj) -> i32 {
    ext::it_80273130(gobj)
}

pub unsafe extern "C" fn itLipstickspore_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    itResetVelocity(ip);
}

pub unsafe extern "C" fn itLipstickspore_UnkMotion1_Coll(_gobj: *mut Item_GObj) -> i32 {
    0
}

pub unsafe fn itLipstickSpore_Logic37_DmgDealt(_arg0: *mut Item_GObj) -> i32 {
    1
}

pub unsafe fn itLipstickSpore_Logic37_Clanked(_arg0: *mut Item_GObj) -> i32 {
    1
}

pub unsafe fn itLipstickSpore_Logic37_HitShield(_arg0: *mut Item_GObj) -> i32 {
    1
}

pub unsafe fn itLipstickSpore_Logic37_Absorbed(_arg0: *mut Item_GObj) -> i32 {
    1
}

pub unsafe fn itLipstickSpore_Logic37_Reflected(gobj: *mut Item_GObj) -> i32 {
    let ip = GET_ITEM(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::it_80273030(gobj);
    HSD_JObjSetRotationZ(jobj, ext::atan2f((*ip).x40_vel.y, (*ip).x40_vel.x));
    0
}

pub unsafe fn itLipstickSpore_Logic37_ShieldBounced(gobj: *mut Item_GObj) -> i32 {
    ext::itColl_BounceOffShield(gobj);
    0
}

pub unsafe fn itLipstickSpore_Logic37_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
