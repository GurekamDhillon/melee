//! Native port of src/melee/it/kinds/itfreeze.c: the Freeze item (Freezie): rolls along the ground at
//! `x0` (flipping at walls), falls, can be picked up and thrown, and in state 4 slides to a stop under a
//! deceleration before it is released (state 5 is the 90-frame wait). Callees in other TUs go through
//! `ext` (unported: panics "TODO"); the header inlines the TU uses (`Item_InitSpawnCommonFields`,
//! `itResetVelocity`, the jobj.h accessors) are ported here. Checked against the C (the whole TU, shipping
//! contraction) by tests/tu_itfreeze.rs.
//!
//! One fused multiply-add site (`fmadds`, f32): `it_8028EDBC`, `xDD4.freeze.x4.x * attrs->xC + x0`.
//! `ABS` keeps the sign of a zero; `(f32) it_804D6D28->x30_lifetime` converts a u32.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DAA8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DAA8(a0) };
    fn it_8026E414(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E414(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_80276100(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::itmaplib::it_80276100(a0, a1 as _) };
    fn it_802762B0(a0: *mut Item) => unsafe { crate::item::it_802762b0(&mut *a0) };
    fn it_80276408(a0: *mut HSD_GObj, a1: *mut CollData, a2: *mut Vec) => unsafe { crate::tu::itmaplib::it_80276408(a0, a1, a2 as _) };
    fn it_8027649C(a0: *mut HSD_GObj) -> f32 => unsafe { crate::itkinds10::it_8027649c(&*((*a0).user_data as *mut Item)) };
    fn it_802E37A4(a0: *mut HSD_GObj) => unsafe { crate::tu::itwhitebea::it_802E37A4(a0 as _) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::it_3F14::it_804D6D28;
}
// END GENERATED EXT

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

/// `-1.0f * x` as the console computes it: a real `fmuls` (a NaN operand keeps its sign). A literal
/// `-1.0 * x` is folded to a negation by the compiler, which flips the sign of a NaN.
#[inline(never)]
fn mul_m1(x: f32) -> f32 {
    core::hint::black_box(-1.0f32) * x
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

/// `itResetVelocity` (it/inlines.h): zeroes the velocity.
#[inline]
pub unsafe fn itResetVelocity(ip: *mut Item) {
    melee_core::ct!("itResetVelocity");
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
}

/// `ItemStateTable it_803F5D28[6]`.
pub static it_803F5D28: [ItemStateTable; 6] = [
    ItemStateTable { anim_id: 0, animated: Some(itFreeze_UnkMotion0_Anim), physics_updated: Some(itFreeze_UnkMotion0_Phys), collided: Some(itFreeze_UnkMotion0_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itFreeze_UnkMotion3_Anim), physics_updated: Some(itFreeze_UnkMotion1_Phys), collided: Some(itFreeze_UnkMotion1_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itFreeze_UnkMotion2_Anim), physics_updated: None, collided: None },
    ItemStateTable { anim_id: 1, animated: Some(itFreeze_UnkMotion3_Anim), physics_updated: Some(itFreeze_UnkMotion3_Phys), collided: Some(itFreeze_UnkMotion3_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itFreeze_UnkMotion4_Anim), physics_updated: Some(itFreeze_UnkMotion4_Phys), collided: Some(itFreeze_UnkMotion4_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itFreeze_UnkMotion5_Anim), physics_updated: Some(itFreeze_UnkMotion5_Phys), collided: Some(itFreeze_UnkMotion5_Coll) },
];

/// `Item_GObj* it_8028EB88(Item_GObj* ref, Vec3* pos, f32 facing_dir, f32 decel)`: spawns the Freeze item at
/// `pos` (at the reference item's position when there is one) and starts it sliding with `decel`.
pub unsafe extern "C" fn it_8028EB88(ref_gobj: *mut Item_GObj, pos: *mut Vec, facing_dir: f32, decel: f32) -> *mut Item_GObj {
    melee_core::ct!("it_8028EB88");
    let mut spawn: SpawnItem = core::mem::zeroed();
    let mut ip: *mut Item = null_mut(); // uninitialised in the C when `ref_gobj` is NULL (zero in the game build)
    if !ref_gobj.is_null() {
        ip = item_of(ref_gobj);
    }
    spawn.kind = It_Kind_Freeze;
    spawn.prev_pos = *pos;
    if !ip.is_null() {
        spawn.pos = (*ip).pos;
    } else {
        spawn.pos = spawn.prev_pos;
    }
    Item_InitSpawnCommonFields(&mut spawn, null_mut(), facing_dir, true);
    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        it_3F14_Logic17_Spawned(gobj);
        it_8028F434(gobj, decel, ref_gobj);
        return gobj;
    }
    null_mut()
}

/// `void it_8028EC98(Item_GObj*, f32 vel)`.
pub unsafe extern "C" fn it_8028EC98(gobj: *mut Item_GObj, vel: f32) {
    melee_core::ct!("it_8028EC98");
    let ip = item_of(gobj);
    (*ip).xDD4_itemVar.freeze.x10 = vel;
    ext::it_80275158(gobj, (*ext::it_804D6D28).x30_lifetime as f32);
}

/// `void it_8028ECE0(Item_GObj*)`: forgets the linked item.
pub unsafe extern "C" fn it_8028ECE0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028ECE0");
    (*item_of(gobj)).xDD4_itemVar.freeze.unk_1C = null_mut();
}

/// `Item* it_8028ECF0(Item_GObj*, Vec3*)`: copies the position out.
pub unsafe extern "C" fn it_8028ECF0(gobj: *mut Item_GObj, v: *mut Vec) -> *mut Item {
    melee_core::ct!("it_8028ECF0");
    let ip = item_of(gobj);
    (*v).x = (*ip).pos.x;
    (*v).y = (*ip).pos.y;
    (*v).z = (*ip).pos.z;
    ip
}

/// `void itFreeze_Logic17_Destroyed(Item_GObj*)`: releases the linked item.
pub unsafe extern "C" fn itFreeze_Logic17_Destroyed(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreeze_Logic17_Destroyed");
    let ip = item_of(gobj);
    if !(*ip).xDD4_itemVar.freeze.unk_1C.is_null() {
        ext::it_802E37A4((*ip).xDD4_itemVar.freeze.unk_1C);
        (*ip).xDD4_itemVar.freeze.unk_1C = null_mut();
    }
}

/// `void it_3F14_Logic17_Spawned(Item_GObj*)`: faces the centre, clears the vars, enters state 1.
pub unsafe extern "C" fn it_3F14_Logic17_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("it_3F14_Logic17_Spawned");
    let ip = item_of(gobj);
    if (*ip).pos.x > 0.0 {
        (*ip).facing_dir = -1.0;
    } else {
        (*ip).facing_dir = 1.0;
    }
    (*ip).xD5C = 0;
    (*ip).xDD4_itemVar.freeze.x0 = 0.0;
    (*ip).xDD4_itemVar.freeze.x4.x = 0.0;
    (*ip).xDD4_itemVar.freeze.x4.y = 1.0;
    (*ip).xDD4_itemVar.freeze.x4.z = 0.0;
    (*ip).xDD4_itemVar.freeze.unk_1C = null_mut();
    it_8028F1D8(gobj);
}

/// `void it_8028EDBC(Item_GObj*)`: rolling speed `x0` along the ground: accelerates with the slope's `x4.x`,
/// clamps to the attribute maximum, turns the item to the rolling direction.
pub unsafe extern "C" fn it_8028EDBC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028EDBC");
    let ip = item_of(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itUnkAttributes;

    if (*ip).ground_or_air != GA_Air {
        let fz = &mut (*ip).xDD4_itemVar.freeze;
        if c_abs(fz.x4.x) > 0.1 {
            if fz.x0 == 0.0 {
                if fz.x4.x > 0.0 {
                    (*ip).facing_dir = 1.0;
                } else {
                    (*ip).facing_dir = -1.0;
                }
                fz.x0 = (*attrs).x4_float * (*ip).facing_dir;
            } else {
                fz.x0 = fmadds(fz.x4.x, (*attrs).xC, fz.x0);
            }
        } else if c_abs(fz.x0) < (*attrs).x4_float {
            fz.x0 = (*attrs).x4_float * (*ip).facing_dir;
        } else if fz.x0 > 0.0 {
            fz.x0 -= (*attrs).x10;
        } else {
            fz.x0 += (*attrs).x10;
        }
        if fz.x0 > (*attrs).x8 {
            fz.x0 = (*attrs).x8;
        } else if fz.x0 < -(*attrs).x8 {
            fz.x0 = -(*attrs).x8;
        }
        if (fz.x0 > 0.0 && (*ip).facing_dir < 0.0) || (fz.x0 < 0.0 && (*ip).facing_dir > 0.0) {
            (*ip).facing_dir *= -1.0;
        }
        (*ip).x40_vel.x = fz.x0 * fz.x4.y;
    }
}

/// `void it_8028EF34(Item_GObj*)`: back to rolling (state 0) at speed `x0`.
pub unsafe extern "C" fn it_8028EF34(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028EF34");
    let ip = item_of(gobj);
    (*ip).x40_vel.x = (*ip).xDD4_itemVar.freeze.x0;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    ext::it_8026B390(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

/// `bool itFreeze_UnkMotion0_Anim(Item_GObj*)`.
pub unsafe extern "C" fn itFreeze_UnkMotion0_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreeze_UnkMotion0_Anim");
    0
}

/// `void itFreeze_UnkMotion0_Phys(Item_GObj*)`.
pub unsafe extern "C" fn itFreeze_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreeze_UnkMotion0_Phys");
    it_8028EDBC(gobj);
}

/// `bool itFreeze_UnkMotion0_Coll(Item_GObj*)`: map collision while rolling: floor slope, wall turn-around, jobj roll.
pub unsafe extern "C" fn itFreeze_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreeze_UnkMotion0_Coll");
    let ip = item_of(gobj);
    let jobj = jobj_of(gobj);
    let cd = &raw mut (*ip).x378_itemColl;

    ext::it_8026D62C(gobj, Some(it_8028F1D8));
    if (*ip).ground_or_air != GA_Air {
        if (*cd).env_flags & 0x18000 != 0 {
            ext::it_80276408(gobj, cd, &mut (*ip).xDD4_itemVar.freeze.x4);
        } else {
            (*ip).xDD4_itemVar.freeze.x4.x = 0.0;
        }
        {
            let env = (*cd).env_flags;
            if (env & 0x3F) != 0 || (env & 0xFC0) != 0 {
                (*ip).facing_dir = -(*ip).facing_dir;
                (*ip).xDD4_itemVar.freeze.x0 = -(*ip).xDD4_itemVar.freeze.x0;
                (*ip).x40_vel.x = (*ip).xDD4_itemVar.freeze.x0 * (*ip).xDD4_itemVar.freeze.x4.y;
            }
        }
        if (*ip).facing_dir == -1.0 {
            let r = ext::it_8027649C(gobj);
            ji::HSD_JObjSetRotationZ(jref(jobj), r, &mut Dirty);
        } else {
            let r = mul_m1(ext::it_8027649C(gobj));
            ji::HSD_JObjSetRotationZ(jref(jobj), r, &mut Dirty);
        }
    }
    0
}

/// `void it_8028F1D8(Item_GObj*)`: enters the falling state (1).
pub unsafe extern "C" fn it_8028F1D8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028F1D8");
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

/// `bool itFreeze_UnkMotion3_Anim(Item_GObj*)`.
pub unsafe extern "C" fn itFreeze_UnkMotion3_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreeze_UnkMotion3_Anim");
    0
}

/// `void itFreeze_UnkMotion1_Phys(Item_GObj*)`: gravity.
pub unsafe extern "C" fn itFreeze_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreeze_UnkMotion1_Phys");
    let ip = item_of(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

/// `bool itFreeze_UnkMotion1_Coll(Item_GObj*)`: landing.
pub unsafe extern "C" fn itFreeze_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreeze_UnkMotion1_Coll");
    let it = item_of(gobj);
    let cd = &raw mut (*it).x378_itemColl;

    ext::it_8026E414(gobj, Some(it_8028EF34));
    if (*it).ground_or_air != GA_Air && ((*cd).env_flags & (Collide_FloorPush | Collide_FloorHug)) != 0 {
        ext::it_80276408(gobj, cd, &mut (*it).xDD4_itemVar.freeze.x4);
    }
    0
}

/// `void itFreeze_Logic17_PickedUp(Item_GObj*)`: releases the linked item, enters the held state (2).
pub unsafe extern "C" fn itFreeze_Logic17_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreeze_Logic17_PickedUp");
    let item = (*gobj).user_data as *mut Item;
    let linked = (*item).xDD4_itemVar.freeze.unk_1C;
    if !linked.is_null() {
        ext::it_802E37A4(linked);
        (*item).xDD4_itemVar.freeze.unk_1C = null_mut();
    }
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
}

/// `bool itFreeze_UnkMotion2_Anim(Item_GObj*)`.
pub unsafe extern "C" fn itFreeze_UnkMotion2_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreeze_UnkMotion2_Anim");
    0
}

/// `void itFreeze_Logic17_Dropped(Item_GObj*)`.
pub unsafe extern "C" fn itFreeze_Logic17_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreeze_Logic17_Dropped");
    ext::Item_80268E5C(gobj, 1, 6);
}

/// `void itFreeze_Logic17_Thrown(Item_GObj*)`.
pub unsafe extern "C" fn itFreeze_Logic17_Thrown(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreeze_Logic17_Thrown");
    ext::Item_80268E5C(gobj, 3, 6);
}

/// `void itFreeze_UnkMotion3_Phys(Item_GObj*)`: gravity.
pub unsafe extern "C" fn itFreeze_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreeze_UnkMotion3_Phys");
    let ip = (*gobj).user_data as *mut Item;
    let it_attr = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*it_attr).x10_fall_speed, (*it_attr).x14_fall_speed_max);
}

/// `bool itFreeze_UnkMotion3_Coll(Item_GObj*)`.
pub unsafe extern "C" fn itFreeze_UnkMotion3_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreeze_UnkMotion3_Coll");
    if ext::it_8026DAA8(gobj) != 0 {
        return 1;
    }
    0
}

/// `bool itFreeze_Logic17_DmgDealt(Item_GObj*)`.
pub unsafe extern "C" fn itFreeze_Logic17_DmgDealt(arg0: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreeze_Logic17_DmgDealt");
    1
}

/// `bool itFreeze_Logic17_Clanked(Item_GObj*)`.
pub unsafe extern "C" fn itFreeze_Logic17_Clanked(arg0: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreeze_Logic17_Clanked");
    1
}

/// `bool itFreeze_Logic17_HitShield(Item_GObj*)`.
pub unsafe extern "C" fn itFreeze_Logic17_HitShield(arg0: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreeze_Logic17_HitShield");
    1
}

/// `bool itFreeze_Logic17_Absorbed(Item_GObj*)`.
pub unsafe extern "C" fn itFreeze_Logic17_Absorbed(arg0: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreeze_Logic17_Absorbed");
    1
}

/// `bool itFreeze_Logic17_Reflected(Item_GObj*)`.
pub unsafe extern "C" fn itFreeze_Logic17_Reflected(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreeze_Logic17_Reflected");
    ext::it_80273030(gobj)
}

/// `bool itFreeze_Logic17_ShieldBounced(Item_GObj*)`.
pub unsafe extern "C" fn itFreeze_Logic17_ShieldBounced(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreeze_Logic17_ShieldBounced");
    ext::itColl_BounceOffShield(gobj)
}

/// `bool itFreeze_Logic17_DmgReceived(Item_GObj*)`.
pub unsafe extern "C" fn itFreeze_Logic17_DmgReceived(arg0: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreeze_Logic17_DmgReceived");
    1
}

/// `void it_8028F434(Item_GObj*, f32 vel, Item_GObj* ref)`: starts the slide with deceleration `|vel|`, linked to `ref`.
pub unsafe extern "C" fn it_8028F434(gobj: *mut Item_GObj, mut vel: f32, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028F434");
    let ip = item_of(gobj);
    itResetVelocity(ip);
    if vel < 0.0 {
        vel = -vel;
    }
    (*ip).xDD4_itemVar.freeze.x14 = vel;
    (*ip).xDD4_itemVar.freeze.unk_1C = ref_gobj;
    ext::it_8026B390(gobj);
    ext::it_802762B0(ip);
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
}

/// `bool itFreeze_UnkMotion4_Anim(Item_GObj*)`.
pub unsafe extern "C" fn itFreeze_UnkMotion4_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreeze_UnkMotion4_Anim");
    0
}

/// `void itFreeze_UnkMotion4_Phys(Item_GObj*)`: applies the stored initial speed, then decelerates by `x14` to zero.
pub unsafe extern "C" fn itFreeze_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreeze_UnkMotion4_Phys");
    let ip = item_of(gobj);
    {
        let x10 = (*ip).xDD4_itemVar.freeze.x10;
        if x10 != 0.0 {
            (*ip).x40_vel.x = x10;
            (*ip).xDD4_itemVar.freeze.x10 = 0.0 * 0.0;
        }
    }

    let mut decel = (*ip).xDD4_itemVar.freeze.x14;
    let abs_decel = c_abs(decel);

    let vel = (*ip).x40_vel.x;
    let abs_vel = c_abs(vel);

    if abs_vel < abs_decel {
        (*ip).x40_vel.x = 0.0;
        return;
    }

    if vel > 0.0 {
        if (*ip).xDD4_itemVar.freeze.x14 < 0.0 {
            decel = -decel;
        }
        (*ip).x40_vel.x -= decel;
    } else {
        if decel < 0.0 {
            decel = -decel;
        }
        (*ip).x40_vel.x += decel;
    }
}

/// `itFreeze_ClearLinkedItem` (static inline): releases the linked item.
pub unsafe fn itFreeze_ClearLinkedItem(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreeze_ClearLinkedItem");
    let ip = item_of(gobj);
    let linked = (*ip).xDD4_itemVar.freeze.unk_1C;
    if !linked.is_null() {
        ext::it_802E37A4(linked);
        (*ip).xDD4_itemVar.freeze.unk_1C = null_mut();
    }
}

/// `itFreeze_ResetToMotion0` (static inline): back to rolling at speed `x0`.
pub unsafe fn itFreeze_ResetToMotion0(gobj: *mut Item_GObj) {
    melee_core::ct!("itFreeze_ResetToMotion0");
    let ip = item_of(gobj);
    (*ip).x40_vel.x = (*ip).xDD4_itemVar.freeze.x0;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    ext::it_8026B390(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

/// `bool itFreeze_UnkMotion4_Coll(Item_GObj*)`: during the slide: wall hit -> wait state, no link -> roll again.
pub unsafe extern "C" fn itFreeze_UnkMotion4_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreeze_UnkMotion4_Coll");
    let ip = item_of(gobj);
    let jobj = jobj_of(gobj);
    let cd = &raw mut (*ip).x378_itemColl;

    ext::it_8026D62C(gobj, Some(it_8028F1D8));
    if (*ip).ground_or_air == GA_Air {
        itFreeze_ClearLinkedItem(gobj);
    } else {
        let env = (*cd).env_flags;
        if (env & 0x3F) != 0 || (env & 0xFC0) != 0 {
            it_8028F7C8(gobj);
            itFreeze_ClearLinkedItem(gobj);
            return 0;
        }
        if (*ip).xDD4_itemVar.freeze.unk_1C.is_null() {
            itFreeze_ResetToMotion0(gobj);
            return 0;
        }
        if (*ip).facing_dir == -1.0 {
            let r = ext::it_8027649C(gobj);
            ji::HSD_JObjSetRotationZ(jref(jobj), r, &mut Dirty);
        } else {
            let r = mul_m1(ext::it_8027649C(gobj));
            ji::HSD_JObjSetRotationZ(jref(jobj), r, &mut Dirty);
        }
    }
    0
}

/// `void it_8028F7C8(Item_GObj*)`: stops and enters the 90-frame wait (state 5).
pub unsafe extern "C" fn it_8028F7C8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028F7C8");
    let ip = item_of(gobj);
    itResetVelocity(ip);
    ext::it_8026B390(gobj);
    (*ip).xDD4_itemVar.freeze.x18 = 90;
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
}

/// `bool itFreeze_UnkMotion5_Anim(Item_GObj*)`.
pub unsafe extern "C" fn itFreeze_UnkMotion5_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreeze_UnkMotion5_Anim");
    0
}

/// `void itFreeze_UnkMotion5_Phys(Item_GObj*)`.
pub unsafe extern "C" fn itFreeze_UnkMotion5_Phys(gobj: *mut Item_GObj) { melee_core::ct!("itFreeze_UnkMotion5_Phys");}

/// `bool itFreeze_UnkMotion5_Coll(Item_GObj*)`: counts the wait down, then rolls again.
pub unsafe extern "C" fn itFreeze_UnkMotion5_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFreeze_UnkMotion5_Coll");
    let ip = item_of(gobj);
    ext::it_8026D62C(gobj, Some(it_8028F1D8));
    if (*ip).xDD4_itemVar.freeze.x18 <= 0 {
        let ip2 = item_of(gobj);
        (*ip2).x40_vel.x = (*ip2).xDD4_itemVar.freeze.x0;
        (*ip2).x40_vel.z = 0.0;
        (*ip2).x40_vel.y = 0.0;
        ext::it_8026B390(gobj);
        ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    } else {
        (*ip).xDD4_itemVar.freeze.x18 -= 1;
    }
    0
}

/// `void itFreeze_Logic17_EvtUnk(Item_GObj*, Item_GObj*)`: removes the gobj interaction references.
pub unsafe extern "C" fn itFreeze_Logic17_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itFreeze_Logic17_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}

/// `void it_8028F8E4(Item_GObj*)`: marks the flag, stops, and runs the map probe from the current position.
pub unsafe extern "C" fn it_8028F8E4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028F8E4");
    let ip = item_of(gobj);
    (*ip).xDCC_flag.set_b3(1);
    ext::it_8026B390(gobj);
    (*ip).x378_itemColl.cur_pos = (*ip).pos;
    let mut pos = (*ip).pos;
    ext::it_80276100(gobj, &mut pos);
}

/// `void it_8028F968(Item_GObj*)`.
pub unsafe extern "C" fn it_8028F968(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028F968");
    let ip = item_of(gobj);
    ext::it_80273454(gobj);
    (*ip).xDCC_flag.set_b3(0);
    ext::it_8026B3A8(gobj);
}

/// `void it_8028F9B8(Item_GObj*)`: destroys the item.
pub unsafe extern "C" fn it_8028F9B8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028F9B8");
    ext::Item_8026A8EC(gobj);
}
