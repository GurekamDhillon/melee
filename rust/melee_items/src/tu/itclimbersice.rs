//! Native port of src/melee/it/kinds/itclimbersice.c (Ice Climbers' ice block): the spawn
//! helpers, the four motion state callbacks (`it_803F7668`), the Logic90 hooks and the collision
//! callbacks. Callees in other TUs go through `ext` (named like the C, settable by the tests and
//! the engine); callees already ported in this crate are called directly. The C `bool` callbacks
//! are `-> bool`; the state table stores `i32`-returning wrappers (`*_p`), since `HSD_GObjPredicate`
//! is `int (*)(HSD_GObj*)`. Checked by tests/tu_itclimbersice.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

/// `ITEM_ANIM_UPDATE` (src/melee/it/forward.h): `1 << 1`.
pub const ITEM_ANIM_UPDATE: u32 = 1 << 1;

/// One argument of a variadic effect spawn (`efSync_Spawn` / `efAsync_Spawn` are C varargs; the port keeps the
/// C type of each extra argument so the eventual implementation can read it back).
#[derive(Clone, Copy)]
pub enum EfArg {
    Int(i32),
    Ptr(*mut c_void),
    F32P(*const f32),
    Vec3P(*const Vec3),
}

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
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn Item_8026A8EC(a0: *mut HSD_GObj);
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj, a5: &[EfArg]);
    fn efLib_DestroyAll(a0: *mut HSD_GObj);
    fn ftKb_SpecialNIc_80108CE8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn ftPp_Init_8011F16C(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32;
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj);
    fn it_8026B3A8(a0: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec);
    fn it_8026BD6C(a0: *mut HSD_GObj);
    fn it_8026BD84(a0: *mut HSD_GObj);
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_8026DAA8(a0: *mut HSD_GObj) -> i32;
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_80272460(a0: *mut HitCapsule, a1: u32, a2: *mut HSD_GObj);
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32);
    fn it_80272980(a0: *mut HSD_GObj);
    fn it_80272F7C(a0: *mut HSD_JObj, a1: f32);
    fn it_80273030(a0: *mut HSD_GObj) -> i32;
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32;
    fn it_802750F8(a0: *mut HSD_GObj);
    fn it_80275158(a0: *mut HSD_GObj, a1: f32);
    fn it_80276308(a0: *mut HSD_GObj) -> i32;
    fn it_80276CB8(a0: *mut HSD_GObj);
    fn it_8027770C(a0: *mut HSD_GObj) -> i32;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut attrs: *mut ItemAttr = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D28: *mut ItemCommonData = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

spawn_inl!();
spawn_parent_inl!();

/// `it_803F7668`'s predicate form: the C callbacks return `bool`, the table stores `int`.
macro_rules! pred_i32 {
    ($($name:ident => $f:ident;)*) => { $( pub unsafe extern "C" fn $name(g: *mut HSD_GObj) -> i32 { $f(g) as i32 } )* };
}

#[inline]
unsafe fn item_of(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `ip->xC4_article_data->x4_specialAttributes` as a `T`.
#[inline]
unsafe fn sa<'a, T>(ip: *mut Item) -> &'a T {
    &*((*(*ip).xC4_article_data).x4_specialAttributes as *const T)
}

/// `HSD_JObjGetChild(jobj)` (NULL-safe).
#[inline]
unsafe fn HSD_JObjGetChild(j: *mut HSD_JObj) -> *mut HSD_JObj {
    if j.is_null() {
        null_mut()
    } else {
        (*j).child
    }
}

/// `Item_TickLifetime` (src/melee/it/inlines.h, static inline).
#[inline]
unsafe fn Item_TickLifetime(ip: *mut Item) -> bool {
    if (*ip).xD44_lifeTimer <= 0.0 {
        return true;
    }
    (*ip).xD44_lifeTimer -= 1.0;
    false
}

/// `itResetVelocity` (src/melee/it/inlines.h, static inline).
#[inline]
unsafe fn itResetVelocity(ip: *mut Item) {
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.z = 0.0;
}

/// `Item_ApplyFallingPhysics` (src/melee/it/inlines.h, static inline).
#[inline]
unsafe fn Item_ApplyFallingPhysics(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attrs = &*(*ip).xCC_item_attr;
    ext::it_80272860(gobj, attrs.x10_fall_speed, attrs.x14_fall_speed_max);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
}

/// C `ABS(x)`: `(x) < 0 ? -(x) : (x)`.
#[inline]
fn c_abs(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

// ---- spawn helpers (static inline in the C) ----------------------------------------------------

/// `itClimbersIce_sub_x4` (static inline): `lifeTimer -= attr->x4`.
pub unsafe fn itClimbersIce_sub_x4(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attr = sa::<itClimbersIceAttributes>(ip);
    (*ip).xD44_lifeTimer -= attr.x4;
}

/// `itClimbersice_Spawn` (static inline): the ice's effect, picked by kind.
pub unsafe fn itClimbersice_Spawn(gobj: *mut HSD_GObj, spawn1: u32, spawn2: u32) {
    let child = HSD_JObjGetChild((*gobj).hsd_obj as *mut HSD_JObj);
    let ip = item_of(gobj);
    if (*ip).kind == It_Kind_IceClimber_Ice {
        ext::efAsync_Spawn(gobj, &mut (*ip).xBC0 as *mut _ as *mut c_void, 3, spawn1, child, &[EfArg::F32P(&(*ip).facing_dir)]);
    } else {
        ext::efAsync_Spawn(gobj, &mut (*ip).xBC0 as *mut _ as *mut c_void, 3, spawn2, child, &[EfArg::F32P(&(*ip).facing_dir)]);
    }
}

/// `itClimbersice_Spawn2` (static inline): the ice's child effect (no facing argument).
pub unsafe fn itClimbersice_Spawn2(gobj: *mut HSD_GObj) {
    let child = HSD_JObjGetChild((*gobj).hsd_obj as *mut HSD_JObj);
    let ip = item_of(gobj);
    if (*ip).kind == It_Kind_IceClimber_Ice {
        ext::efAsync_Spawn(gobj, &mut (*ip).xBC0 as *mut _ as *mut c_void, 0, 0x4E9, child, &[]);
    } else {
        ext::efAsync_Spawn(gobj, &mut (*ip).xBC0 as *mut _ as *mut c_void, 0, 0x4AE, child, &[]);
    }
}

// ---- spawn / lifecycle -----------------------------------------------------------------------

/// `it_802C1590` (itclimbersice.c): spawn an ice block for `parent_gobj`.
pub unsafe extern "C" fn it_802C1590(parent_gobj: *mut HSD_GObj, pos: *mut Vec3, kind: enum_t, facing_dir: f32) -> *mut HSD_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = kind;
    Item_InitSpawnPositionFromParent(&mut spawn, parent_gobj, pos);
    Item_InitSpawnCommonFields(&mut spawn, parent_gobj, facing_dir, true);
    let item_gobj = ext::Item_80268B18(&mut spawn);
    if !item_gobj.is_null() {
        let ip = item_of(item_gobj);
        (*ip).xDD4_itemVar.climbersice.x0 = (*ip).owner as *mut Item_GObj;
        (*ip).xDD4_itemVar.climbersice.set_x8_b0(0);
        (*ip).xDD4_itemVar.climbersice.x4 = (*(*ip).xCC_item_attr).x60_scale;
        itClimbersice_Spawn2(item_gobj);
        it_802C1A58(item_gobj);
        ext::db_80225DD8(item_gobj, parent_gobj);
        ext::it_8026B3A8(item_gobj);
        ext::it_802750F8(item_gobj);
    }
    item_gobj
}

/// `it_802C16F8` (itclimbersice.c): launch the ice along its facing direction.
pub unsafe extern "C" fn it_802C16F8(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let sa = sa::<itClimbersIceAttributes>(ip);
    (*ip).x40_vel.x = sa.x10 * (*ip).facing_dir;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    ext::it_80275158(gobj, sa.x0);
    (*ip).xDD4_itemVar.climbersice.set_x8_b0(1);
    ext::it_80272980(gobj);
    itClimbersice_Spawn(gobj, 0x4ea, 0x4af);
    it_802C1AE4(gobj);
}

/// `it_802C17DC` (itclimbersice.c): `Item_8026A8EC`.
pub unsafe extern "C" fn it_802C17DC(gobj: *mut HSD_GObj) {
    ext::Item_8026A8EC(gobj);
}

/// `it_2725_Logic90_Destroyed` (itclimbersice.c): release the owning climber's ice reference.
pub unsafe extern "C" fn it_2725_Logic90_Destroyed(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    if !(*ip).xDD4_itemVar.climbersice.x0.is_null() {
        if (*ip).kind == It_Kind_IceClimber_Ice {
            ext::ftPp_Init_8011F16C((*ip).xDD4_itemVar.climbersice.x0 as *mut HSD_GObj, gobj);
        } else {
            ext::ftKb_SpecialNIc_80108CE8((*ip).xDD4_itemVar.climbersice.x0 as *mut HSD_GObj, gobj);
        }
    }
    (*ip).xDD4_itemVar.climbersice.x0 = null_mut();
}

/// `it_802C1854` (itclimbersice.c): bounce off a slope while sliding; true once it stops.
pub unsafe extern "C" fn it_802C1854(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    let coll: *mut CollData = &mut (*ip).x378_itemColl;
    let sa = sa::<itClimbersIceAttributes>(ip);
    let mut result = false;

    if (*ip).x378_itemColl.env_flags & 0x18000 != 0 {
        let normal_x = (*coll).floor.normal.x;
        let normal_sign: i32 = if normal_x < 0.0 { -1 } else { 1 };
        let vel_sign: i32 = if (*ip).x40_vel.x < 0.0 { -1 } else { 1 };
        let delta: f32;

        if vel_sign != normal_sign {
            delta = sa.x1C * (normal_x * sa.x14);
        } else {
            delta = sa.x18 * (normal_x * sa.x14);
        }
        (*ip).x40_vel.x += delta;

        if c_abs((*ip).x40_vel.x) < sa.x24 {
            ext::it_8026BD84(gobj);
            ext::it_8026BD6C(gobj);
            it_802C1950(gobj);
            ext::efLib_DestroyAll(gobj);
            result = true;
        }
    }
    result
}

/// `it_802C1950` (itclimbersice.c): zero the velocity and enter motion 0.
pub unsafe extern "C" fn it_802C1950(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    itResetVelocity(ip);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

// ---- motion 0 --------------------------------------------------------------------------------

/// `itClimbersice_UnkMotion0_Anim`: shrink the ice's scale while it is armed.
pub unsafe extern "C" fn itClimbersice_UnkMotion0_Anim(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let sa = sa::<itClimbersIceAttributes>(ip);
    let child = if jobj.is_null() { null_mut() } else { (*jobj).child };

    if (*ip).xDD4_itemVar.climbersice.x8_b0() != 0 {
        (*ip).xDD4_itemVar.climbersice.x4 *= sa.x28;
        ext::it_80272F7C(child, (*ip).xDD4_itemVar.climbersice.x4);
        if (*ip).xDD4_itemVar.climbersice.x4 < 0.01 {
            return true;
        }
    }
    false
}

/// `itClimbersice_UnkMotion0_Phys` (empty).
pub unsafe extern "C" fn itClimbersice_UnkMotion0_Phys(_gobj: *mut HSD_GObj) {}

/// `itClimbersice_UnkMotion0_Coll`.
pub unsafe extern "C" fn itClimbersice_UnkMotion0_Coll(gobj: *mut HSD_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(it_802C1A58));
    ext::it_80276CB8(gobj);
    false
}

/// `it_802C1A58`: enter motion 1.
pub unsafe extern "C" fn it_802C1A58(gobj: *mut HSD_GObj) {
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

// ---- motion 1 --------------------------------------------------------------------------------

/// `itClimbersice_UnkMotion1_Anim` (returns false).
pub unsafe extern "C" fn itClimbersice_UnkMotion1_Anim(_gobj: *mut HSD_GObj) -> bool {
    false
}

/// `itClimbersice_UnkMotion1_Phys`: fall.
pub unsafe extern "C" fn itClimbersice_UnkMotion1_Phys(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attrs = &*(*ip).xCC_item_attr;
    ext::it_80272860(gobj, attrs.x10_fall_speed, attrs.x14_fall_speed_max);
}

/// `itClimbersice_UnkMotion1_Coll`.
pub unsafe extern "C" fn itClimbersice_UnkMotion1_Coll(gobj: *mut HSD_GObj) -> bool {
    ext::it_8026E15C(gobj, Some(it_802C1950));
    false
}

/// `it_802C1AE4`: spawn the shatter effects and enter motion 2.
pub unsafe extern "C" fn it_802C1AE4(gobj: *mut HSD_GObj) {
    itClimbersice_Spawn(gobj, 0x4eb, 0x4b0);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
}

// ---- motion 2 --------------------------------------------------------------------------------

/// `itClimbersice_UnkMotion2_Anim`: tick the lifetime.
pub unsafe extern "C" fn itClimbersice_UnkMotion2_Anim(gobj: *mut HSD_GObj) -> bool {
    Item_TickLifetime(item_of(gobj))
}

/// `itClimbersice_Phys_inline` (static inline): the sliding hitbox, `dmg = |vx * x30| + x2C`.
pub unsafe fn itClimbersice_Phys_inline(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let sa = sa::<itClimbersIceAttributes>(ip);
    let dmg = (c_abs((*ip).x40_vel.x * (sa.x30 as f32)) as u32).wrapping_add(sa.x2C as u32);
    ext::it_80272460(&mut (*ip).x5D4_hitboxes[0].hit, dmg, gobj);
}

/// `itClimbersice_UnkMotion2_Phys`: slide while `it_802C1854` is still false.
pub unsafe extern "C" fn itClimbersice_UnkMotion2_Phys(gobj: *mut HSD_GObj) {
    if it_802C1854(gobj) {
        itClimbersice_Phys_inline(gobj);
    }
}

/// `itClimbersice_Coll` (static inline): true when the ice should be destroyed.
pub unsafe fn itClimbersice_Coll(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    let sa = sa::<itClimbersIceAttributes>(ip);
    if ext::it_80276308(gobj) != 0 {
        if c_abs((*ip).x40_vel.x) <= sa.xC {
            ext::it_8027770C(gobj);
            ext::it_80272980(gobj);
            itClimbersIce_sub_x4(gobj);
        } else {
            return true;
        }
    }
    false
}

/// `itClimbersice_UnkMotion2_Coll`.
pub unsafe extern "C" fn itClimbersice_UnkMotion2_Coll(gobj: *mut HSD_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(fn_802C1D44));
    ext::it_80276CB8(gobj);
    itClimbersice_Coll(gobj)
}

/// `fn_802C1D44`: enter motion 3 and clear the effects.
pub unsafe extern "C" fn fn_802C1D44(gobj: *mut HSD_GObj) {
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE);
    ext::efLib_DestroyAll(gobj);
}

// ---- motion 3 --------------------------------------------------------------------------------

/// `itClimbersice_UnkMotion3_Anim`: tick the lifetime.
pub unsafe extern "C" fn itClimbersice_UnkMotion3_Anim(gobj: *mut HSD_GObj) -> bool {
    Item_TickLifetime(item_of(gobj))
}

/// `itClimbersice_UnkMotion3_Phys`: fall, then the hitbox.
pub unsafe extern "C" fn itClimbersice_UnkMotion3_Phys(gobj: *mut HSD_GObj) {
    Item_ApplyFallingPhysics(gobj);
    itClimbersice_Phys_inline(gobj);
}

/// `itClimbersice_UnkMotion3_Coll`.
pub unsafe extern "C" fn itClimbersice_UnkMotion3_Coll(gobj: *mut HSD_GObj) -> bool {
    if ext::it_8026DAA8(gobj) & 1 != 0 {
        itClimbersice_Spawn(gobj, 0x4eb, 0x4b0);
        ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
    }
    itClimbersice_Coll(gobj)
}

// ---- Logic90 hooks ---------------------------------------------------------------------------

/// `itClimbersIce_Logic90_DmgDealt`: always true.
pub unsafe extern "C" fn itClimbersIce_Logic90_DmgDealt(_arg0: *mut HSD_GObj) -> bool {
    true
}

/// `itClimbersIce_Logic90_Reflected`: `it_80273030`.
pub unsafe extern "C" fn itClimbersIce_Logic90_Reflected(gobj: *mut HSD_GObj) -> i32 {
    ext::it_80273030(gobj)
}

/// `itClimbersIce_Logic90_Clanked`: always true.
pub unsafe extern "C" fn itClimbersIce_Logic90_Clanked(_arg0: *mut HSD_GObj) -> bool {
    true
}

/// `it_2725_Logic90_HitShield`: bounce off the victim while slow enough, else break.
pub unsafe extern "C" fn it_2725_Logic90_HitShield(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    let attr = &*(*ip).xCC_item_attr;
    let sa = sa::<itClimbersIceAttributes>(ip);

    if c_abs((*ip).x40_vel.x) <= sa.xC {
        ext::itColl_BounceOffVictim(gobj);
        ext::it_80272980(gobj);
        let scale = attr.x58;
        (*ip).x40_vel.x *= scale;
        (*ip).x40_vel.y *= scale;
        (*ip).x40_vel.z *= scale;
        itClimbersIce_sub_x4(gobj);
        if (*ip).xD44_lifeTimer < sa.x8 {
            return true;
        }
    } else {
        return true;
    }
    false
}

/// `itClimbersIce_Logic90_Absorbed`: always true.
pub unsafe extern "C" fn itClimbersIce_Logic90_Absorbed(_arg0: *mut HSD_GObj) -> bool {
    true
}

/// `itClimbersIce_Logic90_ShieldBounced`: `itColl_BounceOffShield`.
pub unsafe extern "C" fn itClimbersIce_Logic90_ShieldBounced(gobj: *mut HSD_GObj) -> i32 {
    ext::itColl_BounceOffShield(gobj)
}

/// `itClimbersIce_Logic90_EvtUnk`: forget the climber reference, then `it_8026B894`.
pub unsafe extern "C" fn itClimbersIce_Logic90_EvtUnk(gobj: *mut HSD_GObj, ref_gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    if (*ip).xDD4_itemVar.climbersice.x0 as *mut HSD_GObj == ref_gobj {
        (*ip).xDD4_itemVar.climbersice.x0 = null_mut();
    }
    ext::it_8026B894(gobj, ref_gobj);
}

// ---- state table -----------------------------------------------------------------------------

pred_i32! {
    itClimbersice_UnkMotion0_Anim_p => itClimbersice_UnkMotion0_Anim;
    itClimbersice_UnkMotion0_Coll_p => itClimbersice_UnkMotion0_Coll;
    itClimbersice_UnkMotion1_Anim_p => itClimbersice_UnkMotion1_Anim;
    itClimbersice_UnkMotion1_Coll_p => itClimbersice_UnkMotion1_Coll;
    itClimbersice_UnkMotion2_Anim_p => itClimbersice_UnkMotion2_Anim;
    itClimbersice_UnkMotion2_Coll_p => itClimbersice_UnkMotion2_Coll;
    itClimbersice_UnkMotion3_Anim_p => itClimbersice_UnkMotion3_Anim;
    itClimbersice_UnkMotion3_Coll_p => itClimbersice_UnkMotion3_Coll;
}

/// `it_803F7668[]` (itclimbersice.c).
pub static it_803F7668: [ItemStateTable; 4] = [
    ItemStateTable { anim_id: -1, animated: Some(itClimbersice_UnkMotion0_Anim_p), physics_updated: Some(itClimbersice_UnkMotion0_Phys), collided: Some(itClimbersice_UnkMotion0_Coll_p) },
    ItemStateTable { anim_id: -1, animated: Some(itClimbersice_UnkMotion1_Anim_p), physics_updated: Some(itClimbersice_UnkMotion1_Phys), collided: Some(itClimbersice_UnkMotion1_Coll_p) },
    ItemStateTable { anim_id: 0, animated: Some(itClimbersice_UnkMotion2_Anim_p), physics_updated: Some(itClimbersice_UnkMotion2_Phys), collided: Some(itClimbersice_UnkMotion2_Coll_p) },
    ItemStateTable { anim_id: 0, animated: Some(itClimbersice_UnkMotion3_Anim_p), physics_updated: Some(itClimbersice_UnkMotion3_Phys), collided: Some(itClimbersice_UnkMotion3_Coll_p) },
];
