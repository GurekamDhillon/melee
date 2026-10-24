//! Port of src/melee/it/kinds/itmasterhandbullet.c: Master Hand's bullet (logic table 83; `it_803F9388`).
//! `masterhandbullet.x0` is the remaining lifetime in frames. Callees of unported TUs go through `ext`
//! (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itmasterhandbullet.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE60() -> u32;
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec) -> *mut c_void;
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_80274C60(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80274c60(&mut *((*a0).user_data as *mut Item)) };
    fn it_802750F8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802750F8(a0) };
    fn it_8027AE34(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027AE34(a0) as _ };
    fn it_8027AF50(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027AF50(a0) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

kinds_jobj_inl!();

/// `ItemStateTable it_803F9388[]`
pub static mut it_803F9388: [ItemStateTable; 2] = [
    st_row(0, pred_b(itMasterhandbullet_UnkMotion1_Anim), Some(itMasterhandbullet_UnkMotion1_Phys), pred_b(itMasterhandbullet_UnkMotion1_Coll)),
    st_row(1, pred_b(itMasterhandbullet_UnkMotion1_Anim), Some(itMasterhandbullet_UnkMotion1_Phys), pred_b(itMasterhandbullet_UnkMotion1_Coll)),
];

/// `it_802F0AE0_sub` (static inline)
#[inline]
unsafe fn it_802F0AE0_sub(gobj: *mut Item_GObj, msid: enum_t, parent_gobj: *mut Item_GObj) {
    if msid != 0 {
        it_802F0D2C(gobj);
    } else {
        it_802F0BE8(gobj);
    }
    ext::db_80225DD8(gobj, parent_gobj);
    ext::it_802750F8(gobj);
}

/// 2F0AE0
pub unsafe extern "C" fn it_802F0AE0(
    gobj: *mut Item_GObj,
    prev_pos: *mut Vec3,
    pos: *mut Vec3,
    kind: ItemKind,
    msid: i32,
    facing_dir: f32,
    vel_x: f32,
    vel_y: f32,
) {
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = kind;
    spawn.prev_pos = *prev_pos;
    spawn.pos = *pos;
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.x = vel_x * facing_dir;
    spawn.vel.y = vel_y;
    spawn.vel.z = 0.0;
    spawn.x0_parent_gobj = gobj;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = ext::Item_8026AE60() as i32;
    let spawned = ext::Item_80268B18(&mut spawn);
    it_802F0AE0_sub(spawned, msid, gobj);
}

pub unsafe extern "C" fn itMasterHandBullet_Logic85_EvtUnk(gobj: *mut Item_GObj, arg1: *mut Item_GObj) {
    ext::it_8026B894(gobj, arg1);
}

/// 2F0BE8
pub unsafe extern "C" fn it_802F0BE8(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMasterHandBulletAttributes;
    let mut quad: Quaternion = core::mem::zeroed();
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    (*ip).on_accessory = Some(it_802F0F04);
    (*ip).xDD4_itemVar.masterhandbullet.x0 = (*attrs).x0;
    ext::it_80274C60(gobj);
    quad.y = (1.5707963267948966f64 * (*ip).facing_dir as f64) as f32;
    HSD_JObjSetRotation(jobj, &mut quad);
}

/// 2F0D2C
pub unsafe extern "C" fn it_802F0D2C(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMasterHandBulletAttributes;
    let mut quad: Quaternion = core::mem::zeroed();
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    (*ip).on_accessory = Some(it_802F0F04);
    (*ip).xDD4_itemVar.masterhandbullet.x0 = (*attrs).x0;
    ext::it_80274C60(gobj);
    quad.y = (1.5707963267948966f64 * (*ip).facing_dir as f64) as f32;
    HSD_JObjSetRotation(jobj, &mut quad);
}

pub unsafe extern "C" fn itMasterhandbullet_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    ext::efSync_Spawn(0x405, gobj, &mut (*ip).pos);
    let temp_f1 = (*ip).xDD4_itemVar.masterhandbullet.x0 - 1.0;
    (*ip).xDD4_itemVar.masterhandbullet.x0 = temp_f1;
    if temp_f1 < 0.0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn itMasterhandbullet_UnkMotion1_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itMasterhandbullet_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E15C(gobj, Some(it_802F0F08));
    false
}

/// 2F0F04
pub unsafe extern "C" fn it_802F0F04(_gobj: *mut HSD_GObj) {}

/// 2F0F08
pub unsafe extern "C" fn it_802F0F08(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itMasterHandBullet_Logic83_DmgDealt(_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itMasterHandBullet_Logic83_Reflected(gobj: *mut Item_GObj) -> bool {
    ext::it_8027AE34(gobj) != 0
}

pub unsafe extern "C" fn itMasterHandBullet_Logic83_Clanked(_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itMasterHandBullet_Logic83_Absorbed(_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itMasterHandBullet_Logic83_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    ext::it_8027AF50(gobj) != 0
}

pub unsafe extern "C" fn itMasterHandBullet_Logic83_HitShield(_gobj: *mut Item_GObj) -> bool {
    true
}
