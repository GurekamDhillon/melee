//! Port of src/melee/it/kinds/itnesspkfire.c: Ness's PK Fire shot (logic table 23). Callees of unported TUs go
//! through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itnesspkfire.rs.
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
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn itNesspkfirepillar_802AA494(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: *mut Vec, a3: f32) => unsafe { crate::tu::itnesspkfirepillar::itNesspkfirepillar_802AA494(a0, a1, a2 as _, a3) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026E058(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026E058(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32);
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

/// `ItemStateTable it_803F6B28[]`
pub static mut it_803F6B28: [ItemStateTable; 1] = [st_row(0, pred_b(itNesspkfire_UnkMotion0_Anim), None, pred_b(itNesspkfire_UnkMotion0_Coll))];

/// `Item_InitSpawnPositionFromParent` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitSpawnPositionFromParent(spawn: *mut SpawnItem, parent: *mut HSD_GObj, pos: *mut Vec3) {
    (*spawn).prev_pos = *pos;
    (*spawn).prev_pos.z = 0.0;
    ext::it_8026BB68(parent, &mut (*spawn).pos);
}

/// 2AA054: spawns a PK Fire shot.
pub unsafe extern "C" fn it_802AA054(gobj: *mut Item_GObj, pos: *mut Vec3, vel: *mut Vec3, facing_dir: f32, angle: f32) {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = It_Kind_Ness_PKFire;
    Item_InitSpawnPositionFromParent(&mut spawn, gobj, pos);
    spawn.vel = *vel;
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.x0_parent_gobj = gobj;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;

    {
        let fire = ext::Item_80268B18(&mut spawn);
        if !fire.is_null() {
            let jobj = GET_JOBJ(fire);
            ext::db_80225DD8(fire, gobj);
            it_802AA1D8(fire);
            HSD_JObjSetRotationZ(jobj, angle);
        }
    }
}

pub unsafe extern "C" fn it_802AA1D8(item_gobj: *mut Item_GObj) {
    let item = GET_ITEM(item_gobj);
    let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itNessPKFirepillarAttributes;
    ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE as u32);
    ext::it_80275158(item_gobj, (*attrs).x0);
}

pub unsafe extern "C" fn itNesspkfire_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = (*gobj).user_data as *mut Item;
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn itNesspkfire_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E058(gobj) != 0
}

pub unsafe extern "C" fn it_2725_Logic23_DmgDealt(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itNessPKFirepillarAttributes;
    let mut pos: Vec3 = (*ip).pos;
    pos.y += (*attrs).x4;
    ext::itNesspkfirepillar_802AA494(gobj, (*ip).owner, &mut pos, (*ip).facing_dir);
    true
}

pub unsafe extern "C" fn it_2725_Logic23_Clanked(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itNessPKFirepillarAttributes;
    let mut pos: Vec3 = (*ip).pos;
    pos.y += (*attrs).x4;
    ext::itNesspkfirepillar_802AA494(gobj, (*ip).owner, &mut pos, (*ip).facing_dir);
    true
}

pub unsafe extern "C" fn itNessPKFire_Logic23_Absorbed(_arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itNessPKFire_Logic23_HitShield(_arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itNessPKFire_Logic23_Reflected(gobj: *mut Item_GObj) -> bool {
    ext::it_80273030(gobj) != 0
}

pub unsafe extern "C" fn it_2725_Logic23_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    let jobj = GET_JOBJ(gobj);
    let rot_z = HSD_JObjGetRotationZ(jobj);
    HSD_JObjSetRotationZ(jobj, -rot_z);
    ext::itColl_BounceOffShield(gobj) != 0
}

pub unsafe extern "C" fn it_802AA474(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
