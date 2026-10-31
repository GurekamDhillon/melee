//! Port of src/melee/it/kinds/itstarrodstar.c: the star shot of the Star Rod (logic table 36).
//! `starrodstar.x0` selects the strong variant, `x4` is the per-frame spin. Callees of unported TUs go
//! through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itstarrodstar.rs.
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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026DFB0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DFB0(a0) };
    fn it_80272460(a0: *mut HitCapsule, a1: u32, a2: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_80272460(a0, a1, a2) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80274740(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274740(a0) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
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

/// `StarRodStarAttrs` (local to itstarrodstar.c)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct StarRodStarAttrs {
    pub x0: f32,
    pub x4: f32,
    pub x8: f32,
    pub xC: f32,
    pub x10: f32,
    pub x14: i32,
    pub x18: i32,
    pub x1C: f32,
}

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

/// `ItemStateTable it_803F6530[]`
pub static mut it_803F6530: [ItemStateTable; 1] = [
    st_row(0, pred_b(itStarrodstar_UnkMotion0_Anim), Some(itStarrodstar_UnkMotion0_Phys), pred_b(itStarrodstar_UnkMotion0_Coll)),
];

/// 2988E4
pub unsafe extern "C" fn it_802988E4(gobj: *mut Fighter_GObj, vec: *mut Vec, arg2: i32, arg3: f32) {
    melee_core::ct!("it_802988E4");
    let mut si: SpawnItem = core::mem::zeroed();
    si.kind = It_Kind_StarRod_Star;
    Item_InitSpawnPositionFromParent(&mut si, gobj, vec);
    Item_InitSpawnCommonFields(&mut si, gobj, arg3, true);
    let n = ext::Item_80268B18(&mut si);
    if !n.is_null() {
        let ip = GET_ITEM(n);
        (*ip).xDD4_itemVar.starrodstar.x0 = arg2;
        it_802989C8(n, gobj);
        ext::db_80225DD8(n, gobj);
    }
}

/// 2989C8
pub unsafe extern "C" fn it_802989C8(ig: *mut Item_GObj, fg: *mut Fighter_GObj) {
    melee_core::ct!("it_802989C8");
    let ip = GET_ITEM(ig);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut StarRodStarAttrs;
    ext::it_8026B3A8(ig);
    ext::it_80274740(ig);
    ext::Item_80268E5C(ig, 0, ITEM_ANIM_UPDATE);
    let hit: *mut HitCapsule = &raw mut (*ip).x5D4_hitboxes[0].hit;
    if (*ip).xDD4_itemVar.starrodstar.x0 != 0 {
        ext::it_80275158(ig, (*attrs).x10);
        (*ip).x40_vel.x = (*ip).facing_dir * ((*attrs).x0 * (*attrs).x8);
        ext::it_80272460(hit, (*attrs).x18 as u32, ig);
    } else {
        ext::it_80275158(ig, (*attrs).xC);
        (*ip).x40_vel.x = (*ip).facing_dir * ((*attrs).x0 * (*attrs).x4);
        ext::it_80272460(hit, (*attrs).x14 as u32, ig);
    }
    (*ip).xDD4_itemVar.starrodstar.x4 = (*attrs).x1C * (*ip).facing_dir;
    (*ip).xDCE_flag.anon_0.set_b7(0);
}

pub unsafe extern "C" fn itStarrodstar_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itStarrodstar_UnkMotion0_Anim");
    let ip = GET_ITEM(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        return true;
    }
    HSD_JObjAddRotationZ(jobj, (*ip).xDD4_itemVar.starrodstar.x4);
    false
}

pub unsafe extern "C" fn itStarrodstar_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itStarrodstar_UnkMotion0_Phys");}

pub unsafe extern "C" fn itStarrodstar_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itStarrodstar_UnkMotion0_Coll");
    if ext::it_8026DFB0(gobj) != 0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn itStarRodStar_Logic36_DmgDealt(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itStarRodStar_Logic36_DmgDealt");
    true
}

pub unsafe extern "C" fn itStarRodStar_Logic36_Clanked(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itStarRodStar_Logic36_Clanked");
    true
}

pub unsafe extern "C" fn itStarRodStar_Logic36_HitShield(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itStarRodStar_Logic36_HitShield");
    true
}

pub unsafe extern "C" fn itStarRodStar_Logic36_Absorbed(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itStarRodStar_Logic36_Absorbed");
    true
}

pub unsafe extern "C" fn itStarRodStar_Logic36_Reflected(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itStarRodStar_Logic36_Reflected");
    let ip = GET_ITEM(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::it_80273030(gobj);
    HSD_JObjSetRotationZ(jobj, ext::atan2f((*ip).x40_vel.y, (*ip).x40_vel.x));
    false
}

pub unsafe extern "C" fn itStarRodStar_Logic36_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itStarRodStar_Logic36_ShieldBounced");
    let ip = GET_ITEM(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::itColl_BounceOffShield(gobj);
    HSD_JObjSetRotationZ(jobj, ext::atan2f((*ip).x40_vel.y, (*ip).x40_vel.x));
    false
}

pub unsafe extern "C" fn itStarRodStar_Logic36_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itStarRodStar_Logic36_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}
