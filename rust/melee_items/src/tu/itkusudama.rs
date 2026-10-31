//! Native port of src/melee/it/kinds/itkusudama.c: the Kusudama (Pokemon Stadium-style "piñata" gift
//! ball) item: spawn, the swing / hit / break states, the item-spawn burst `it_80289BE8`, the
//! hit-or-break dice `it_8028A114` and the state table `it_803F59F8`. Callees in other TUs go through
//! `ext` (hooks; unset panics "TODO"). Checked against the C (the whole TU, shipping contraction) by
//! tests/tu_itkusudama.rs.
//!
//! Header inlines compiled into the TU: `HSD_JObjMtxIsDirty` / `HSD_JObjGetRotationX|Y` /
//! `HSD_JObjSetRotationX|Y` (jobj.h, with their `HSD_ASSERT`s), `Item_InitSpawnPosition`,
//! `Item_InitSpawnCommonFields`, `Item_StopAndEnterState`, `itResetVelocity`,
//! `Item_ApplyFallingPhysics` (it/inlines.h, kinds/inlines.h), `GET_ITEM`.
//!
//! Float math: the contracted sites are the burst velocities in `it_80289BE8_inline`
//! (`vel_scale * HSD_Randf() - vel_scale * 0.5f`, `(vel_scale * 0.5f) * HSD_Randf() - vel_scale * 0.25f`;
//! Gekko `fmsubs`, the left product fused). `it_80289BE8_spawn_random` falls off its end when
//! `it_8026F3AC()` is -1; the PPC build then returns r3 (-1), which is what this port (and the
//! oracle, patched to `return rand_kind;`) returns.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmsubs;
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
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B5C(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B5C(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efSync_Spawn_N(a0: i32, a1: *mut HSD_GObj, a2: &[*mut c_void]) -> *mut c_void;
    fn ftLib_80086960(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ftlib::ftLib_80086960(a0) as _ };
    fn ftLib_800874CC(a0: *mut HSD_GObj, a1: *mut c_void, a2: i32);
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB20(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BB20(a0) };
    fn it_8026BD24(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd24(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026C704() -> i32 => unsafe { crate::tu::itspawn::it_8026C704() as _ };
    fn it_8026D324(a0: i32) -> i32 => unsafe { crate::tu::itspawn::it_8026D324(a0) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026F3AC() -> i32 => unsafe { crate::tu::itdrop::it_8026F3AC() };
    fn it_8026F53C(a0: *mut HSD_GObj, a1: *mut Vec, a2: i32) => unsafe { crate::tu::itdrop::it_8026F53C(a0, a1 as _, a2) };
    fn it_8026F5C8(a0: *mut HSD_GObj, a1: i32, a2: *mut Vec) -> *mut HSD_GObj => unsafe { crate::tu::itdrop::it_8026F5C8(a0, a1, a2 as _) };
    fn it_8026F6BC(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: i32) -> *mut HSD_GObj => unsafe { crate::tu::itdrop::it_8026F6BC(a0, a1 as _, a2 as _, a3) };
    fn it_8026F8B4(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: i32) -> i32 => unsafe { crate::tu::itdrop::it_8026F8B4(a0, a1 as _, a2 as _, a3) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272A3C(a0: *mut HSD_JObj) => unsafe { crate::tu::it_2725::it_80272A3C(a0) };
    fn it_80272C08(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272C08(a0) };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_80274ED8() => unsafe { crate::tu::it_2725::it_80274ED8() };
    fn it_8027518C(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_8027518C(a0) };
    fn it_802751D8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_802751d8(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275444(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275444(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762B0(a0: *mut Item) => unsafe { crate::item::it_802762b0(&mut *a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80276308(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80276308(&mut *((*a0).user_data as *mut Item)) };
    fn it_802763E0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::helpers::it_802763e0(&mut *((*a0).user_data as *mut Item)) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::it_3F14::it_804D6D28;
    pub static mut result: i32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

const M_PI: f64 = 3.14159265358979323846;
const M_PI_F: f32 = 3.14159265358979323846f32;
const M_TAU_F: f32 = 6.283185307179586f32;
const M_PI_2_F: f32 = M_PI_F / 2.0f32;

#[inline]
unsafe fn get_item(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

#[inline]
unsafe fn get_jobj(g: *mut HSD_GObj) -> *mut HSD_JObj {
    (*g).hsd_obj as *mut HSD_JObj
}

#[inline]
unsafe fn attrs(ip: *mut Item) -> *mut itKusudamaAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itKusudamaAttributes
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

/// `HSD_JObjSetMtxDirty(jobj)` (jobj.h macro).
#[inline]
unsafe fn HSD_JObjSetMtxDirtyMacro(jobj: *mut HSD_JObj) {
    if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

/// `HSD_JObjGetRotationX` (jobj.h).
#[inline]
unsafe fn HSD_JObjGetRotationX(jobj: *mut HSD_JObj) -> f32 {
    hsd_assert(!jobj.is_null(), "jobj");
    (*jobj).rotate.x
}

/// `HSD_JObjGetRotationY` (jobj.h).
#[inline]
unsafe fn HSD_JObjGetRotationY(jobj: *mut HSD_JObj) -> f32 {
    hsd_assert(!jobj.is_null(), "jobj");
    (*jobj).rotate.y
}

/// `HSD_JObjSetRotationX` (jobj.h).
#[inline]
unsafe fn HSD_JObjSetRotationX(jobj: *mut HSD_JObj, x: f32) {
    hsd_assert(!jobj.is_null(), "jobj");
    hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
    (*jobj).rotate.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirtyMacro(jobj);
    }
}

/// `HSD_JObjSetRotationY` (jobj.h).
#[inline]
unsafe fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, y: f32) {
    hsd_assert(!jobj.is_null(), "jobj");
    hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
    (*jobj).rotate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirtyMacro(jobj);
    }
}

/// `itResetVelocity` (it/inlines.h).
#[inline]
pub unsafe fn itResetVelocity(ip: *mut Item) {
    melee_core::ct!("itResetVelocity");
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
}

/// `Item_StopAndEnterState` (kinds/inlines.h).
#[inline]
unsafe fn Item_StopAndEnterState(gobj: *mut Item_GObj, ip: *mut Item, msid: i32) {
    itResetVelocity(ip);
    ext::it_8026B390(gobj);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE as _);
}

/// `Item_ApplyFallingPhysics` (it/inlines.h).
#[inline]
unsafe fn Item_ApplyFallingPhysics(gobj: *mut Item_GObj) {
    let a = (*get_item(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*a).x10_fall_speed, (*a).x14_fall_speed_max);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
}

/// `Item_InitSpawnPosition` (kinds/inlines.h).
#[inline]
unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec3, on_plane: bool) {
    (*spawn).prev_pos = *pos;
    if on_plane {
        (*spawn).prev_pos.z = 0.0;
    }
    (*spawn).pos = (*spawn).prev_pos;
}

/// `Item_InitSpawnCommonFields` (kinds/inlines.h).
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

pub static mut it_803F59F8: [ItemStateTable; 9] = [
    ItemStateTable { anim_id: -1, animated: Some(itKusudama_UnkMotion0_Anim), physics_updated: Some(itKusudama_UnkMotion0_Phys), collided: Some(itKusudama_UnkMotion0_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itKusudama_UnkMotion1_Anim), physics_updated: Some(itKusudama_UnkMotion1_Phys), collided: Some(itKusudama_UnkMotion1_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itKusudama_UnkMotion2_Anim), physics_updated: Some(itKusudama_UnkMotion2_Phys), collided: Some(itKusudama_UnkMotion2_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itKusudama_UnkMotion3_Anim), physics_updated: Some(itKusudama_UnkMotion3_Phys), collided: Some(itKusudama_UnkMotion3_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itKusudama_UnkMotion4_Anim), physics_updated: Some(itKusudama_UnkMotion4_Phys), collided: None },
    ItemStateTable { anim_id: 2, animated: Some(itKusudama_UnkMotion6_Anim), physics_updated: Some(itKusudama_UnkMotion6_Phys), collided: Some(itKusudama_UnkMotion5_Coll) },
    ItemStateTable { anim_id: 2, animated: Some(itKusudama_UnkMotion6_Anim), physics_updated: Some(itKusudama_UnkMotion6_Phys), collided: Some(itKusudama_UnkMotion6_Coll) },
    ItemStateTable { anim_id: 3, animated: Some(itKusudama_UnkMotion7_Anim), physics_updated: Some(itKusudama_UnkMotion7_Phys), collided: Some(itKusudama_UnkMotion7_Coll) },
    ItemStateTable { anim_id: 4, animated: Some(itKusudama_UnkMotion8_Anim), physics_updated: Some(itKusudama_UnkMotion8_Phys), collided: Some(itKusudama_UnkMotion8_Coll) },
];

pub unsafe extern "C" fn it_802896CC(arg0: *mut Vec3) -> *mut HSD_GObj {
    melee_core::ct!("it_802896CC");
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = It_Kind_Kusudama as _;
    Item_InitSpawnPosition(&mut spawn, arg0, false);
    Item_InitSpawnCommonFields(&mut spawn, null_mut(), -1.0, true);
    let gobj = ext::Item_80268B5C(&mut spawn);
    if !gobj.is_null() {
        it_8028A190(gobj);
    }
    gobj
}

pub unsafe extern "C" fn itKusudama_Logic4_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itKusudama_Logic4_Spawned");
    let ip = get_item(gobj);
    (*ip).xDCE_flag.anon_0.set_b7(0);
    (*ip).xDAC_itcmd_var0 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDB4_itcmd_var2 = 0;
    (*ip).xDD4_itemVar.kusudama.x0 = 0;
    it_8028A3CC(gobj);
}

pub unsafe extern "C" fn it_802897C8(gobj: *mut Item_GObj, mut angle: f32) {
    melee_core::ct!("it_802897C8");
    let ip = get_item(gobj);
    let jobj = get_jobj(gobj);

    if angle > M_PI_F {
        angle -= M_TAU_F;
    }
    if (*ip).facing_dir == -1.0 {
        angle *= -1.0;
    }
    let rotx = HSD_JObjGetRotationX(jobj);
    let mut f = rotx;
    f -= angle;
    if f < 0.0 {
        f = -f;
    }
    if f > 0.17453292f32 {
        let temp = 0.08726646f32;
        if rotx < angle {
            angle = rotx + temp;
        } else {
            angle = rotx - temp;
        }
    }
    HSD_JObjSetRotationX(jobj, angle);
}

pub unsafe extern "C" fn it_80289910(gobj: *mut Item_GObj, mut angle: f32) {
    melee_core::ct!("it_80289910");
    let ip = get_item(gobj);
    let jobj = get_jobj(gobj);

    if angle > M_PI_F {
        angle -= M_TAU_F;
    }
    if (*ip).facing_dir == -1.0 {
        angle *= -1.0;
    }
    HSD_JObjSetRotationX(jobj, angle);
}

#[inline]
pub unsafe fn it_8028A544_inline(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028A544_inline");
    let ip = get_item(gobj);
    let jobj = get_jobj(gobj);
    let roty: f32 = if (*ip).facing_dir == 1.0 { 4.694936f32 } else { -4.694936f32 };
    HSD_JObjSetRotationY(jobj, roty);
}

pub unsafe extern "C" fn it_80289A00(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80289A00");
    let jobj = get_jobj(gobj);
    let ip = get_item(gobj);
    let attrs = attrs(ip);

    let mut roty = HSD_JObjGetRotationY(jobj);
    if (*ip).facing_dir == 1.0 {
        if roty > M_PI_2_F + (*attrs).x28 {
            roty -= (*attrs).x28;
        } else {
            roty = M_PI_2_F;
        }
    } else if roty < -M_PI_2_F - (*attrs).x28 {
        roty += (*attrs).x28;
    } else {
        roty = -M_PI_2_F;
    }
    HSD_JObjSetRotationY(jobj, roty);
}

pub unsafe extern "C" fn it_80289B50(gobj: *mut Item_GObj, arg1: i32) {
    melee_core::ct!("it_80289B50");
    let ip = get_item(gobj);
    let attr = attrs(ip);
    if it_8028A114(gobj, (*attr).x4, (*attr).x0, (*attr).x8, (*attr).xC) {
        if arg1 != 0 {
            ext::Item_8026AE84(ip, 0x126, 0x7F, 0x40);
        }
        it_8028A544(gobj);
    } else {
        it_8028AC74(gobj);
    }
}

#[inline]
pub unsafe fn it_80289BE8_inline(gobj: *mut Item_GObj, vel_scale: f32, pos: *mut Vec3, vel: *mut Vec3) {
    melee_core::ct!("it_80289BE8_inline");
    let ip = get_item(gobj);
    *pos = (*ip).pos;
    (*pos).y -= 5.0;
    (*vel).x = fmsubs(vel_scale, ext::HSD_Randf(), vel_scale * 0.5f32);
    (*vel).y = fmsubs(vel_scale * 0.5f32, ext::HSD_Randf(), vel_scale * 0.25f32);
    (*vel).z = 0.0;
}

#[inline]
pub unsafe fn it_80289BE8_spawn_random(gobj: *mut Item_GObj, vel_scale: f32, pos: *mut Vec3, vel: *mut Vec3) -> ItemKind {
    melee_core::ct!("it_80289BE8_spawn_random");
    let rand_kind = ext::it_8026F3AC();
    if rand_kind != -1 {
        it_80289BE8_inline(gobj, vel_scale, pos, vel);
        let spawned_gobj = ext::it_8026F5C8(gobj, rand_kind, pos);
        if !spawned_gobj.is_null() {
            ext::it_8026F53C(spawned_gobj, vel, 1);
            ext::it_80274ED8();
            return rand_kind;
        } else {
            return -1;
        }
    }
    rand_kind
}

#[inline]
pub unsafe fn it_80289BE8_spawn(gobj: *mut Item_GObj, kind: ItemKind, vel_scale: f32, pos: *mut Vec3, vel: *mut Vec3, out_spawned: *mut ItemKind) {
    melee_core::ct!("it_80289BE8_spawn");
    it_80289BE8_inline(gobj, vel_scale, pos, vel);

    let spawned_gobj = ext::it_8026F5C8(gobj, kind, pos);
    if !spawned_gobj.is_null() {
        ext::it_8026F53C(spawned_gobj, vel, 1);
        ext::it_80274ED8();
        *out_spawned = kind;
    }
}

pub unsafe extern "C" fn it_80289BE8(gobj: *mut Item_GObj, arg1: i32, mut food_weight: i32, arg3: i32) {
    melee_core::ct!("it_80289BE8");
    let ip = get_item(gobj);
    let attr = attrs(ip);
    let mut spawned: [ItemKind; 15] = [0; 15];
    let mut pos: Vec3 = core::mem::zeroed();
    let mut vel: Vec3 = core::mem::zeroed();
    let mut count: i32;
    let mut i: i32;

    pos = (*ip).pos;
    pos.y -= 5.0;
    vel.z = 0.0;
    vel.y = 0.0;
    vel.x = 0.0;

    if ext::it_8026F8B4(gobj, &mut pos, &mut vel, 0) != 0 {
        return;
    }

    it_80289BE8_inline(gobj, 0.1f32, &mut pos, &mut vel);
    pos.y -= 5.0;

    if !ext::it_8026F6BC(gobj, &mut pos, &mut vel, 1).is_null() {
        return;
    }

    if ext::it_8026D324(It_Kind_Foods) == 0 {
        food_weight = 0;
    }

    for k in 0..15 {
        spawned[k] = -1;
    }

    i = ext::HSD_Randi(arg1.wrapping_add(food_weight).wrapping_add(arg3));
    // `food_or_random` label: entered by falling out of the else arm or by the two gotos.
    let mut to_food_or_random = false;
    if i < arg1 {
        let kind = (*attr).x10;
        count = (*attr).x14;
        if kind == It_Kind_M_Ball && ext::it_8026C704() == 1 {
            i = ext::HSD_Randi(food_weight.wrapping_add(arg3));
            to_food_or_random = true;
        } else if ext::it_8026D324(kind) == 0 {
            i = ext::HSD_Randi(food_weight.wrapping_add(arg3));
            to_food_or_random = true;
        }
        if !to_food_or_random {
            i = 0;
            while i < count {
                if kind == It_Kind_M_Ball && ext::it_8026C704() == 1 {
                    break;
                }
                it_80289BE8_spawn(gobj, kind, 1.4f32, &mut pos, &mut vel, &mut spawned[i as usize]);
                i += 1;
            }
            while i < count {
                let mut vel: Vec3 = core::mem::zeroed();
                let mut pos: Vec3 = core::mem::zeroed();
                spawned[i as usize] = it_80289BE8_spawn_random(gobj, 1.2f32, &mut pos, &mut vel);
                i += 1;
            }
        }
    } else {
        i -= arg1;
        to_food_or_random = true;
    }
    if to_food_or_random {
        if i < food_weight {
            count = ext::HSD_Randi(5) + 10;
            i = 0;
            while i < count {
                it_80289BE8_spawn(gobj, It_Kind_Foods, 1.8f32, &mut pos, &mut vel, &mut spawned[i as usize]);
                i += 1;
            }
        } else {
            let mut vel: Vec3 = core::mem::zeroed();
            let mut pos: Vec3 = core::mem::zeroed();
            count = ext::HSD_Randi(2) + 3;
            i = 0;
            while i < count {
                spawned[i as usize] = it_80289BE8_spawn_random(gobj, 1.2f32, &mut pos, &mut vel);
                i += 1;
            }
        }
    }
    if ext::ftLib_80086960((*ip).owner) != 0 {
        ext::ftLib_800874CC((*ip).owner, spawned.as_mut_ptr() as *mut c_void, i);
    }
}

pub unsafe extern "C" fn it_8028A114(_gobj: *mut Item_GObj, arg1: i32, arg2: i32, arg3: i32, arg4: i32) -> bool {
    melee_core::ct!("it_8028A114");
    let sum1 = arg1.wrapping_add(arg2);
    let sum2 = arg3.wrapping_add(sum1);
    let rand = ext::HSD_Randi(arg4.wrapping_add(sum2));
    if rand < arg1 {
        return true;
    }
    if rand < sum1 {
        return true;
    }
    if rand < sum2 {
        return true;
    }
    false
}

#[inline]
pub unsafe fn it_8028A190_inline(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028A190_inline");
    let ip = get_item(gobj);
    let mut rotx: f32 = (M_PI / 2.0) as f32;
    let jobj = get_jobj(gobj);
    if (*ip).facing_dir == -1.0 {
        rotx *= -1.0;
    }
    HSD_JObjSetRotationX(jobj, rotx);
}

pub unsafe extern "C" fn it_8028A190(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028A190");
    let ip = get_item(gobj);
    Item_StopAndEnterState(gobj, ip, 0);
    it_8028A190_inline(gobj);
}

pub unsafe extern "C" fn itKusudama_UnkMotion0_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_UnkMotion0_Anim");
    it_802897C8(gobj, (M_PI / 2.0) as f32);
    0
}

pub unsafe extern "C" fn itKusudama_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itKusudama_UnkMotion0_Phys");}

pub unsafe extern "C" fn itKusudama_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_UnkMotion0_Coll");
    ext::it_8026D62C(gobj, Some(it_8028A3CC));
    0
}

pub unsafe extern "C" fn itKusudama_UnkMotion1_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_UnkMotion1_Anim");
    let ip = get_item(gobj);
    if (*ip).xDB4_itcmd_var2 == 1 {
        (*ip).xDB4_itcmd_var2 = 2;
        if ext::HSD_Randi(2) != 0 {
            ext::Item_8026AE84(ip, 0x127, 0x7F, 0x40);
        } else {
            ext::Item_8026AE84(ip, 0x128, 0x7F, 0x40);
        }
    }
    if (*ip).xDB0_itcmd_var1 != 0 {
        it_8028AD44(gobj);
    }
    it_802897C8(gobj, 0.0);
    it_80289A00(gobj);
    0
}

pub unsafe extern "C" fn itKusudama_UnkMotion1_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itKusudama_UnkMotion1_Phys");}

pub unsafe extern "C" fn itKusudama_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_UnkMotion1_Coll");
    ext::it_8026DA08(gobj);
    0
}

pub unsafe extern "C" fn it_8028A3CC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028A3CC");
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as _);
    it_8028A190_inline(gobj);
}

pub unsafe extern "C" fn itKusudama_UnkMotion2_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_UnkMotion2_Anim");
    it_802897C8(gobj, (M_PI / 2.0) as f32);
    0
}

pub unsafe extern "C" fn itKusudama_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itKusudama_UnkMotion2_Phys");
    let ip = get_item(gobj);
    let a = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*a).x10_fall_speed, (*a).x14_fall_speed_max);
}

pub unsafe extern "C" fn itKusudama_UnkMotion2_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_UnkMotion2_Coll");
    ext::it_8026E15C(gobj, Some(it_8028A190));
    0
}

pub unsafe extern "C" fn it_8028A544(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028A544");
    let ip = get_item(gobj);
    let attr = attrs(ip);

    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = (*attr).x18;
    (*ip).xDD4_itemVar.kusudama.x4 = (*attr).x1C as i32;
    (*ip).xDD4_itemVar.kusudama.x0 = 1;
    ext::it_802762BC(ip);
    ext::it_80275444(gobj);
    ext::it_8026BB20(gobj);
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE as _);
    it_8028A544_inline(gobj);
}

#[inline]
pub unsafe fn itKusudama_UnkMotion3_inline(gobj: *mut Item_GObj) {
    melee_core::ct!("itKusudama_UnkMotion3_inline");
    let ip = get_item(gobj);
    let jobj = get_jobj(gobj);

    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    (*ip).xDD4_itemVar.kusudama.x4 = 0x5A;
    (*ip).xDD1_flag.anon_0.set_b1(1);
    ext::it_8026B3A8(gobj);
    let rot_y = HSD_JObjGetRotationY(jobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as _);
    HSD_JObjSetRotationY(jobj, rot_y);
}

pub unsafe extern "C" fn itKusudama_UnkMotion3_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_UnkMotion3_Anim");
    let ip = get_item(gobj);

    (*ip).xDD4_itemVar.kusudama.x4 -= 1;
    if (*ip).xDD4_itemVar.kusudama.x4 <= 0 {
        itKusudama_UnkMotion3_inline(gobj);
    }
    it_802897C8(gobj, 0.0);
    it_80289A00(gobj);
    0
}

pub unsafe extern "C" fn itKusudama_UnkMotion3_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itKusudama_UnkMotion3_Phys");}

pub unsafe extern "C" fn itKusudama_UnkMotion3_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_UnkMotion3_Coll");
    let ip = get_item(gobj);
    let coll: *mut CollData = &raw mut (*ip).x378_itemColl;

    ext::it_8026DA08(gobj);
    if (*coll).env_flags & 0x6000 != 0 {
        itKusudama_UnkMotion3_inline(gobj);
    }
    0
}

pub unsafe extern "C" fn itKusudama_Logic4_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itKusudama_Logic4_PickedUp");
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itKusudama_UnkMotion4_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_UnkMotion4_Anim");
    0
}

pub unsafe extern "C" fn itKusudama_UnkMotion4_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itKusudama_UnkMotion4_Phys");}

pub unsafe extern "C" fn itKusudama_Logic4_Thrown(gobj: *mut Item_GObj) {
    melee_core::ct!("itKusudama_Logic4_Thrown");
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 5, (ITEM_ANIM_UPDATE | ITEM_DROP_UPDATE) as _);
}

pub unsafe extern "C" fn itKusudama_UnkMotion6_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_UnkMotion6_Anim");
    0
}

pub unsafe extern "C" fn itKusudama_UnkMotion6_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itKusudama_UnkMotion6_Phys");
    Item_ApplyFallingPhysics(gobj);
}

#[inline]
pub unsafe fn itKusudama_UnkMotion5_Coll_inline(gobj: *mut Item_GObj) {
    melee_core::ct!("itKusudama_UnkMotion5_Coll_inline");
    let ip = get_item(gobj);
    let attr = attrs(ip);
    if it_8028A114(gobj, (*attr).x4, (*attr).x0, (*attr).x8, (*attr).xC) {
        ext::Item_8026AE84(ip, 0x126, 0x7F, 0x40);
        it_8028A544(gobj);
    } else {
        it_8028AC74(gobj);
    }
}

pub unsafe extern "C" fn itKusudama_UnkMotion5_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_UnkMotion5_Coll");
    let ip = get_item(gobj);
    if ext::it_8026DA08(gobj) != 0 {
        itKusudama_UnkMotion5_Coll_inline(gobj);
    } else if ext::it_802763E0(gobj) != 0 {
        if (*ip).x40_vel.y > 0.0 {
            (*ip).x40_vel.y = 0.0;
        }
    } else if ext::it_80276308(gobj) != 0 {
        itKusudama_UnkMotion5_Coll_inline(gobj);
    }
    0
}

pub unsafe extern "C" fn itKusudama_Logic4_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itKusudama_Logic4_Dropped");
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 6, (ITEM_ANIM_UPDATE | ITEM_DROP_UPDATE) as _);
}

#[inline]
pub unsafe fn itKusudama_UnkMotion6_Coll_inline2(gobj: *mut Item_GObj) {
    melee_core::ct!("itKusudama_UnkMotion6_Coll_inline2");
    it_80289910(gobj, (M_PI / 2.0) as f32);
}

#[inline]
pub unsafe fn itKusudama_UnkMotion6_Coll_inline(gobj: *mut Item_GObj) {
    melee_core::ct!("itKusudama_UnkMotion6_Coll_inline");
    let ip = get_item(gobj);
    Item_StopAndEnterState(gobj, ip, 0);
    itKusudama_UnkMotion6_Coll_inline2(gobj);
}

pub unsafe extern "C" fn itKusudama_UnkMotion6_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_UnkMotion6_Coll");
    let ip = get_item(gobj);
    let attr = attrs(ip);

    if ext::it_8026DA08(gobj) != 0 {
        let mut vel_y = (*ip).x40_vel.y;
        if vel_y < 0.0 {
            vel_y = -vel_y;
        }
        if vel_y > (*attr).x2C {
            itKusudama_UnkMotion5_Coll_inline(gobj);
        } else {
            ext::it_802762B0(ip);
            itKusudama_UnkMotion6_Coll_inline(gobj);
        }
    }
    0
}

pub unsafe extern "C" fn it_8028AC74(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028AC74");
    let ip = get_item(gobj);
    let jobj = get_jobj(gobj);
    ext::it_80272A3C(jobj);
    ext::it_80272C08(gobj);
    ext::it_802756D0(gobj);
    ext::it_8026B3A8(gobj);
    ext::it_8026BD24(gobj);
    ext::it_8027518C(gobj);
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).xDCF_flag.anon_0.set_b2(1);
    (*ip).xDD4_itemVar.kusudama.x0 = 1;
    (*ip).xDD4_itemVar.kusudama.x4 = 0x28;
    ext::it_80275444(gobj);
    ext::Item_80268E5C(gobj, 7, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itKusudama_UnkMotion7_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_UnkMotion7_Anim");
    ext::it_802751D8(gobj)
}

pub unsafe extern "C" fn itKusudama_UnkMotion7_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itKusudama_UnkMotion7_Phys");}

pub unsafe extern "C" fn itKusudama_UnkMotion7_Coll(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_UnkMotion7_Coll");
    0
}

pub unsafe extern "C" fn it_8028AD44(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028AD44");
    let ip = get_item(gobj);
    ext::Item_8026AE84(ip, 0xF8, 0x7F, 0x40);
    ext::it_802756D0(gobj);
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).xDCF_flag.anon_0.set_b2(1);
    (*ip).xDD4_itemVar.kusudama.x0 = 1;
    (*ip).xDD4_itemVar.kusudama.x4 = 0x55;
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 8, ITEM_ANIM_UPDATE as _);
}

#[inline]
pub unsafe fn itKusudama_UnkMotion8_Anim_inline(gobj: *mut Item_GObj) {
    melee_core::ct!("itKusudama_UnkMotion8_Anim_inline");
    let ip = get_item(gobj);
    ext::efSync_Spawn_N(0x433, gobj, &[(*(*ip).xBBC_dynamicBoneTable).bones[3] as *mut c_void]);
    ext::efSync_Spawn_N(0x433, gobj, &[(*(*ip).xBBC_dynamicBoneTable).bones[5] as *mut c_void]);
}

pub unsafe extern "C" fn itKusudama_UnkMotion8_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_UnkMotion8_Anim");
    let ip = get_item(gobj);
    let attrs = attrs(ip);
    if (*ip).xDAC_itcmd_var0 != 0 {
        (*ip).xDAC_itcmd_var0 = 0;
        it_80289BE8(gobj, (*attrs).x4, (*attrs).x0, (*attrs).x8);
        itKusudama_UnkMotion8_Anim_inline(gobj);
    }
    (*ip).xDD4_itemVar.kusudama.x4 -= 1;
    if (*ip).xDD4_itemVar.kusudama.x4 > 0 {
        return 0;
    }
    1
}

pub unsafe extern "C" fn itKusudama_UnkMotion8_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itKusudama_UnkMotion8_Phys");}

pub unsafe extern "C" fn itKusudama_UnkMotion8_Coll(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_UnkMotion8_Coll");
    0
}

#[inline]
pub unsafe fn itKusudama_Logic4_DmgDealt_inline(gobj: *mut Item_GObj) {
    melee_core::ct!("itKusudama_Logic4_DmgDealt_inline");
    let ip = get_item(gobj);
    let attr = attrs(ip);
    if it_8028A114(gobj, (*attr).x4, (*attr).x0, (*attr).x8, (*attr).xC) {
        it_8028A544(gobj);
    } else {
        it_8028AC74(gobj);
    }
}

pub unsafe extern "C" fn itKusudama_Logic4_DmgDealt(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_Logic4_DmgDealt");
    let ip = get_item(gobj);
    if (*ip).xDD4_itemVar.kusudama.x0 == 0 {
        itKusudama_Logic4_DmgDealt_inline(gobj);
    }
    0
}

#[inline]
pub unsafe fn itKusudama_Logic4_Clanked_inline(gobj: *mut Item_GObj) {
    melee_core::ct!("itKusudama_Logic4_Clanked_inline");
    it_80289B50(gobj, 0);
}

pub unsafe extern "C" fn itKusudama_Logic4_Clanked(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_Logic4_Clanked");
    let ip = get_item(gobj);
    if (*ip).xDD4_itemVar.kusudama.x0 == 0 {
        itKusudama_Logic4_Clanked_inline(gobj);
    }
    0
}

pub unsafe extern "C" fn itKusudama_Logic4_HitShield(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_Logic4_HitShield");
    itKusudama_Logic4_Clanked(gobj)
}

pub unsafe extern "C" fn itKusudama_Logic4_Reflected(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_Logic4_Reflected");
    itKusudama_Logic4_Clanked(gobj)
}

pub unsafe extern "C" fn itKusudama_Logic4_DmgReceived(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKusudama_Logic4_DmgReceived");
    let ip = get_item(gobj);
    let attr = attrs(ip);
    if (*ip).xDD4_itemVar.kusudama.x0 == 0 {
        if (*ip).xC9C as f32 >= (*attr).x20 {
            itKusudama_Logic4_DmgDealt_inline(gobj);
        }
    }
    0
}

pub unsafe extern "C" fn itKusudama_Logic4_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itKusudama_Logic4_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}
