//! Native port of src/melee/it/kinds/itcrazyhandbomb.c (Crazy Hand's bomb): the spawn helper, the
//! release/detonate chain, the motion 0 spin (`it_803F93A8`) and the motion 1 callbacks. Callees
//! in other TUs go through `ext`; callees ported in this crate are called directly. `*_p` wrappers
//! return `i32` for the state table (`HSD_GObjPredicate`). Checked by tests/tu_itcrazyhandbomb.rs.
//!
//! Fused sites (shipping contraction): the spin `rot.x += MTXDegToRad(10 * vy)` is
//! `fmadds(10*vy, 0.01745329252, rot.x)`, one per axis.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_core::fp::fmadds;

use crate::itlib::{it_8026b894, it_80273030, it_8026bd24, it_802751d8, it_802756d0};
use crate::itlib3::it_80272860;
use crate::item::it_802762bc;

/// `ITEM_ANIM_UPDATE` (src/melee/it/forward.h): `1 << 1`.
pub const ITEM_ANIM_UPDATE: u32 = 1 << 1;

// BEGIN GENERATED EXT (callees of the C TU that are not ported in this crate yet)
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
    fn Item_80268B18(spawn: *mut SpawnItem) -> *mut HSD_GObj;
    fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing: f32, flag: bool);
    fn Item_80268E5C(gobj: *mut HSD_GObj, msid: i32, flags: u32);
    fn db_80225DD8(item: *mut HSD_GObj, owner: *mut HSD_GObj);
    fn it_802750F8(gobj: *mut HSD_GObj);
    fn it_8027518C(gobj: *mut HSD_GObj);
    fn it_80273454(gobj: *mut HSD_GObj);
    fn it_80272C08(gobj: *mut HSD_GObj);
    fn it_8026E71C(gobj: *mut HSD_GObj, cb: HSD_GObjEvent);
    fn HSD_JObjSetFlagsAll(jobj: *mut HSD_JObj, flags: u32);
    fn HSD_JObjSetRotation(jobj: *mut HSD_JObj, rot: *const Quaternion);
    fn HSD_Randi(max: i32) -> i32;
    }
}
// END GENERATED EXT

/// `JOBJ_HIDDEN` (sysdolphin/baselib/jobj.h).
pub const JOBJ_HIDDEN: u32 = 1 << 4;

/// `MTXDegToRad(a)` (dolphin/mtx.h): `a * 0.01745329252f`.
#[inline]
pub fn MTXDegToRad(a: f32) -> f32 {
    a * 0.01745329252f32
}

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

/// `it_802F0F6C` (itcrazyhandbomb.c): spawn a bomb of `kind` from `owner`.
pub unsafe extern "C" fn it_802F0F6C(owner: *mut HSD_GObj, prev_pos: *mut Vec3, pos: *mut Vec3, kind: ItemKind, facing_dir: f32) {
    let mut spawnitem: SpawnItem = core::mem::zeroed();
    spawnitem.kind = kind;
    spawnitem.prev_pos = *prev_pos;
    spawnitem.pos = *pos;
    ext::Item_InitSpawnCommonFields(&mut spawnitem, owner, facing_dir, true);
    let gobj = ext::Item_80268B18(&mut spawnitem);
    it_802F10F8(gobj);
    ext::db_80225DD8(gobj, owner);
    ext::it_802750F8(gobj);
}

/// `it_802F1030` (itcrazyhandbomb.c): hide and detonate the bomb.
pub unsafe extern "C" fn it_802F1030(gobj: *mut HSD_GObj) {
    ext::HSD_JObjSetFlagsAll((*gobj).hsd_obj as *mut HSD_JObj, JOBJ_HIDDEN);
    it_8026bd24(&mut *item_of(gobj));
    ext::it_8027518C(gobj);
    ext::it_80273454(gobj);
    ext::it_80272C08(gobj);
    it_802756d0(&mut *item_of(gobj));
    it_802F1344(gobj);
}

/// `itCrazyHandBomb_Logic86_EvtUnk`: `it_8026B894`.
pub unsafe extern "C" fn itCrazyHandBomb_Logic86_EvtUnk(gobj: *mut HSD_GObj, ref_gobj: *mut HSD_GObj) {
    it_8026b894(&mut *item_of(gobj), ref_gobj);
}

/// `itCrazyHandBomb_Logic86_DmgDealt`: detonate, true.
pub unsafe extern "C" fn itCrazyHandBomb_Logic86_DmgDealt(gobj: *mut HSD_GObj) -> bool {
    it_802F1344(gobj);
    true
}

/// `itCrazyHandBomb_Logic86_Reflected`: `it_80273030`.
pub unsafe extern "C" fn itCrazyHandBomb_Logic86_Reflected(gobj: *mut HSD_GObj) -> bool {
    it_80273030(&mut *item_of(gobj))
}

/// `it_802F10F8` (itcrazyhandbomb.c): launch the bomb upwards at `x0` velocity and arm it.
pub unsafe extern "C" fn it_802F10F8(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attrs = sa::<itCrazyHandBombAttributes>(ip);
    it_802762bc(&mut *ip);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    (*ip).on_accessory = Some(it_802F1340);
    (*ip).x40_vel.y = attrs.x0;
}

/// `itCrazyhandbomb_UnkMotion0_Anim` (returns false).
pub unsafe extern "C" fn itCrazyhandbomb_UnkMotion0_Anim(_gobj: *mut HSD_GObj) -> bool {
    false
}

/// `itCrazyhandbomb_UnkMotion0_Phys`: drift with the attribute motion and spin one axis.
pub unsafe extern "C" fn itCrazyhandbomb_UnkMotion0_Phys(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attrs = sa::<itCrazyHandBombAttributes>(ip);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    it_80272860(&mut *ip, attrs.x4, attrs.x8);
    let mut rot: Quaternion = (*jobj).rotate;

    let vy = (*ip).x40_vel.y;
    match ext::HSD_Randi(3) {
        0 => rot.x = fmadds(10.0 * vy, 0.01745329252f32, rot.x),
        1 => rot.y = fmadds(10.0 * vy, 0.01745329252f32, rot.y),
        2 => rot.z = fmadds(10.0 * vy, 0.01745329252f32, rot.z),
        _ => {}
    }

    ext::HSD_JObjSetRotation(jobj, &rot);
}

/// `itCrazyhandbomb_UnkMotion0_Coll`: detonate when airborne.
pub unsafe extern "C" fn itCrazyhandbomb_UnkMotion0_Coll(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    if (*ip).ground_or_air == GA_Air {
        ext::it_8026E71C(gobj, Some(it_802F1030));
    }
    false
}

/// `it_802F1340`: the on-accessory callback once the bomb is armed (empty). The C declares it
/// with no parameters; it is stored as an `HSD_GObjEvent`, so it takes the gobj.
pub unsafe extern "C" fn it_802F1340(_gobj: *mut HSD_GObj) {}

/// `it_802F1344`: switch to motion 1 and arm the explosion callback.
pub unsafe extern "C" fn it_802F1344(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    (*ip).on_accessory = Some(it_802F13B0);
}

/// `itCrazyhandbomb_UnkMotion1_Anim`: `it_802751D8`.
pub unsafe extern "C" fn itCrazyhandbomb_UnkMotion1_Anim(gobj: *mut HSD_GObj) -> bool {
    it_802751d8(&mut *item_of(gobj))
}

/// `itCrazyhandbomb_UnkMotion1_Phys` (empty).
pub unsafe extern "C" fn itCrazyhandbomb_UnkMotion1_Phys(_gobj: *mut HSD_GObj) {}

/// `itCrazyhandbomb_UnkMotion1_Coll` (returns false).
pub unsafe extern "C" fn itCrazyhandbomb_UnkMotion1_Coll(_gobj: *mut HSD_GObj) -> bool {
    false
}

/// `it_802F13B0` (empty), the armed-bomb on-accessory callback.
pub unsafe extern "C" fn it_802F13B0(_gobj: *mut HSD_GObj) {}

pred_i32! {
    itCrazyhandbomb_UnkMotion0_Anim_p => itCrazyhandbomb_UnkMotion0_Anim;
    itCrazyhandbomb_UnkMotion0_Coll_p => itCrazyhandbomb_UnkMotion0_Coll;
    itCrazyhandbomb_UnkMotion1_Anim_p => itCrazyhandbomb_UnkMotion1_Anim;
    itCrazyhandbomb_UnkMotion1_Coll_p => itCrazyhandbomb_UnkMotion1_Coll;
}

/// `it_803F93A8[]` (itcrazyhandbomb.c).
pub static it_803F93A8: [ItemStateTable; 2] = [
    ItemStateTable { anim_id: 0, animated: Some(itCrazyhandbomb_UnkMotion0_Anim_p), physics_updated: Some(itCrazyhandbomb_UnkMotion0_Phys), collided: Some(itCrazyhandbomb_UnkMotion0_Coll_p) },
    ItemStateTable { anim_id: 1, animated: Some(itCrazyhandbomb_UnkMotion1_Anim_p), physics_updated: Some(itCrazyhandbomb_UnkMotion1_Phys), collided: Some(itCrazyhandbomb_UnkMotion1_Coll_p) },
];
