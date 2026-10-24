//! Port of src/melee/it/kinds/itsscopebeam.c: the Super Scope shot (logic table 38). `scopebeam.x0` is
//! the charge level (0..=9) that indexes the attribute table.
//! Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU,
//! shipping contraction) by tests/kinds_itsscopebeam.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026D9A0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D9A0(a0) };
    fn it_8026DAA8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DAA8(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn lbVector_RotateAboutUnitAxis(a0: *mut Vec, a1: *mut Vec, a2: f32) => unsafe { melee_runtime::tu::lbvector::lbVector_RotateAboutUnitAxis(a0, a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

/// `Item_InitSpawnCommonFields` (it/kinds/inlines.h, static inline)
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

/// `Item_InitSpawnPositionFromParent` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitSpawnPositionFromParent(spawn: *mut SpawnItem, parent: *mut HSD_GObj, pos: *mut Vec3) {
    (*spawn).prev_pos = *pos;
    (*spawn).prev_pos.z = 0.0;
    ext::it_8026BB68(parent, &raw mut (*spawn).pos);
}

/// `ItemStateTable it_803F6568[]`
pub static mut it_803F6568: [ItemStateTable; 10] = [
    st_row(0, pred_b(itSscopebeam_UnkMotion9_Anim), Some(itSscopebeam_UnkMotion9_Phys), pred_b(itSscopebeam_UnkMotion9_Coll)),
    st_row(1, pred_b(itSscopebeam_UnkMotion9_Anim), Some(itSscopebeam_UnkMotion9_Phys), pred_b(itSscopebeam_UnkMotion9_Coll)),
    st_row(2, pred_b(itSscopebeam_UnkMotion9_Anim), Some(itSscopebeam_UnkMotion9_Phys), pred_b(itSscopebeam_UnkMotion9_Coll)),
    st_row(3, pred_b(itSscopebeam_UnkMotion9_Anim), Some(itSscopebeam_UnkMotion9_Phys), pred_b(itSscopebeam_UnkMotion9_Coll)),
    st_row(4, pred_b(itSscopebeam_UnkMotion9_Anim), Some(itSscopebeam_UnkMotion9_Phys), pred_b(itSscopebeam_UnkMotion9_Coll)),
    st_row(5, pred_b(itSscopebeam_UnkMotion9_Anim), Some(itSscopebeam_UnkMotion9_Phys), pred_b(itSscopebeam_UnkMotion9_Coll)),
    st_row(6, pred_b(itSscopebeam_UnkMotion9_Anim), Some(itSscopebeam_UnkMotion9_Phys), pred_b(itSscopebeam_UnkMotion9_Coll)),
    st_row(7, pred_b(itSscopebeam_UnkMotion9_Anim), Some(itSscopebeam_UnkMotion9_Phys), pred_b(itSscopebeam_UnkMotion9_Coll)),
    st_row(8, pred_b(itSscopebeam_UnkMotion9_Anim), Some(itSscopebeam_UnkMotion9_Phys), pred_b(itSscopebeam_UnkMotion9_Coll)),
    st_row(9, pred_b(itSscopebeam_UnkMotion9_Anim), Some(itSscopebeam_UnkMotion9_Phys), pred_b(itSscopebeam_UnkMotion9_Coll)),
];

/// 298DEC: spawns a beam of charge level `arg2` from the fighter `gobj`.
pub unsafe extern "C" fn it_80298DEC(gobj: *mut Fighter_GObj, vec: *mut Vec, arg2: i32, arg3: f32) {
    let mut si: SpawnItem = core::mem::zeroed();
    si.kind = It_Kind_S_Scope_Beam;
    Item_InitSpawnPositionFromParent(&mut si, gobj, vec);
    Item_InitSpawnCommonFields(&mut si, gobj, arg3, true);
    let n = ext::Item_80268B18(&mut si);
    if !n.is_null() {
        let ip = GET_ITEM(n);
        (*ip).xDD4_itemVar.scopebeam.x0 = arg2;
        it_80298ED0(n, gobj);
        ext::db_80225DD8(n, gobj);
    }
}

/// 298ED0
pub unsafe extern "C" fn it_80298ED0(projectile: *mut HSD_GObj, owner: *mut HSD_GObj) {
    let it = GET_ITEM(projectile);
    let jobj = GET_JOBJ(projectile);
    let attrs = (*(*it).xC4_article_data).x4_specialAttributes as *mut ScopeBeamAttrs;
    let mut scale: Vec3 = core::mem::zeroed();

    (*it).owner = owner;
    let data = ((*attrs).floats.as_mut_ptr()).offset((*it).xDD4_itemVar.scopebeam.x0 as isize);
    ext::it_8026B3A8(projectile);
    match (*it).xDD4_itemVar.scopebeam.x0 {
        0 => ext::Item_80268E5C(projectile, 0, ITEM_ANIM_UPDATE),
        1 => ext::Item_80268E5C(projectile, 1, ITEM_ANIM_UPDATE),
        2 => ext::Item_80268E5C(projectile, 2, ITEM_ANIM_UPDATE),
        3 => ext::Item_80268E5C(projectile, 3, ITEM_ANIM_UPDATE),
        4 => ext::Item_80268E5C(projectile, 4, ITEM_ANIM_UPDATE),
        5 => ext::Item_80268E5C(projectile, 5, ITEM_ANIM_UPDATE),
        6 => ext::Item_80268E5C(projectile, 6, ITEM_ANIM_UPDATE),
        7 => ext::Item_80268E5C(projectile, 7, ITEM_ANIM_UPDATE),
        8 => ext::Item_80268E5C(projectile, 8, ITEM_ANIM_UPDATE),
        9 => ext::Item_80268E5C(projectile, 9, ITEM_ANIM_UPDATE),
        _ => {}
    }

    (*it).x40_vel.x = (*data).velocity * (*it).facing_dir;
    scale.z = (*data).scale;
    scale.y = scale.z;
    scale.x = scale.y;
    HSD_JObjSetScale(jobj, &mut scale);
    ext::it_80275158(projectile, (*data).lifetime);
}

pub unsafe extern "C" fn itSscopebeam_UnkMotion9_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let index = (*ip).xDD4_itemVar.scopebeam.x0;
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut ScopeBeamAttrs;
    let data = ((*attrs).floats.as_mut_ptr()).offset(index as isize);

    if (*ip).xD44_lifeTimer == (*data).lifetime {
        if (*ip).x5D4_hitboxes[0].hit.state >= HitCapsule_Enabled {
            (*ip).x5D4_hitboxes[0].hit.scale = (*ip).x5D4_hitboxes[0].hit.scale * (*data).scale;
        }
    }

    (*ip).xD44_lifeTimer = (*ip).xD44_lifeTimer - 1.0;

    if (*ip).xD44_lifeTimer <= 0.0 {
        (*ip).xD44_lifeTimer = 0.0;
        return true;
    }

    false
}

pub unsafe extern "C" fn itSscopebeam_UnkMotion9_Phys(_gobj: *mut Item_GObj) {}

#[inline]
fn ABS(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

pub unsafe extern "C" fn itSscopebeam_UnkMotion9_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let coll: *mut CollData = &raw mut (*ip).x378_itemColl;
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut ScopeBeamAttrs;
    let mut vel: Vec3 = core::mem::zeroed();
    let mut axis: Vec3 = core::mem::zeroed();
    let saved_pos: Vec3 = (*ip).pos;
    let saved_coll: CollData = *coll;
    let saved_vel_x: f32 = (*ip).x40_vel.x;

    let result = ext::it_8026DAA8(gobj);

    if result & 0xE != 0 {
        return true;
    }

    if (result & 1) != 0 && ((*coll).env_flags & 0x18000) != 0 {
        let angle: f32 = -ext::atan2f((*coll).floor.normal.x, (*coll).floor.normal.y);

        if ABS(angle) <= 1.0f32 * 0.01745329252f32 {
            (*ip).pos = saved_pos;
            *coll = saved_coll;
            ext::it_8026D9A0(gobj);
        } else if ABS(angle) <= (*attrs).x78 {
            vel.x = (*ip).x40_vel.x * (*attrs).x7C;
            vel.z = 0.0;
            vel.y = vel.z;
            axis.y = 0.0;
            axis.x = axis.y;
            axis.z = 1.0;
            let rand = ext::HSD_Randf();
            ext::lbVector_RotateAboutUnitAxis(&mut vel, &mut axis, fmadds(2.0, angle, (*ip).facing_dir * ((*attrs).x78 * rand)));
            (*ip).x40_vel.x = vel.x;
            (*ip).x40_vel.y = vel.y;
            if (*ip).x40_vel.y <= 0.0 {
                (*ip).x40_vel.x = saved_vel_x;
                (*ip).x40_vel.y = 0.0;
            }
        } else {
            return true;
        }
    }

    false
}

pub unsafe extern "C" fn itSScopeBeam_Logic38_DmgDealt(_arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itSScopeBeam_Logic38_Clanked(_arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itSScopeBeam_Logic38_HitShield(_arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itSScopeBeam_Logic38_Absorbed(_arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itSScopeBeam_Logic38_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    ext::itColl_BounceOffShield(gobj) != 0
}

pub unsafe extern "C" fn itSScopeBeam_Logic38_Reflected(gobj: *mut Item_GObj) -> bool {
    ext::it_80273030(gobj) != 0
}

pub unsafe extern "C" fn itSScopeBeam_Logic38_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
