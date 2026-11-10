//! Port of src/melee/it/kinds/itsamusgrapple.c: Samus's grapple beam (logic table 53): the chain of links, its
//! physics for every beam state, and the fighter-side callbacks. Callees of unported TUs go through `ext`
//! (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itsamusgrapple.rs.
//!
//! GameCube bit layouts: `it_802B7160` reads the damage and the `hit_grounded` flag of the packed hitbox
//! record by byte offset (`*(u16*)(data + 2) & 0x3FF`, `(flags + 3)->hit_grounded`); on the GameCube
//! (MSB-first bitfields) those are exactly the `damage` and `hit_grounded` fields, which is what the port
//! reads. The native oracle compiles a copy of the TU with those two reads replaced by the field accesses
//! (`patch` in gen/kinds/specs.py), since native bitfields are LSB-first.
//! Every float expression the shipping compiler contracts is written with `fmadds` (f32) or `mul_add` (f64).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code, unused_assignments)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
use melee_core::fp::Fmadd as _;
#[allow(unused_imports)]
use crate::itkinds7::bisect_value;
#[allow(unused_imports)]
use crate::layout::itEgg_ItemVars;
#[allow(unused_imports)]
use crate::layout::itLizardonAttributes;
#[allow(unused_imports)]
use crate::layout::itPeachTurnipAttributes;
#[allow(unused_imports)]
use crate::layout::it_2E5A_Attrs;
#[allow(unused_imports)]
use crate::layout::ItemPickTable;
#[allow(unused_imports)]
use melee_core::fp::frsqrte;
#[allow(unused_imports)]
use melee_runtime::random::HsdRand;
#[allow(unused_imports)]
use crate::helpers::it_link_lerp;
#[allow(unused_imports)]
use crate::itlib::it_80271534;
#[allow(unused_imports)]
use crate::itlib::it_8026b3a8;
#[allow(unused_imports)]
use crate::itlib::it_8026b894;
#[allow(unused_imports)]
use crate::itlib::it_80274d6c;
#[allow(unused_imports)]
use crate::itlib::it_80274ecc;
#[allow(unused_imports)]
use crate::itlib::it_8027572c;
#[allow(unused_imports)]
use crate::itlib3::it_80272860;
#[allow(unused_imports)]
use crate::itlib3::it_80274658;
#[allow(unused_imports)]
use crate::itlib3::it_80274d04;
#[allow(unused_imports)]
use crate::itlib3::it_802756e0;
#[allow(unused_imports)]
use crate::itlib3::it_8027a344;
#[allow(unused_imports)]
use crate::itkinds::itGShell_Attrs;
#[allow(unused_imports)]
use crate::layout::ftCollisionBox;
#[allow(unused_imports)]
use crate::layout::itLinkHookshotAttributes;
#[allow(unused_imports)]
use crate::layout::itMarilAttributes;
#[allow(unused_imports)]
use crate::layout::itSScopeAttributes;
#[allow(unused_imports)]
use crate::layout::itSword_UnkArticle1;
#[allow(unused_imports)]
use crate::layout::itTaruAttributes;
#[allow(unused_imports)]
use crate::layout::itTaruCann_DatAttrs;
#[allow(unused_imports)]
use crate::layout::itToolsAttributes;
#[allow(unused_imports)]
use crate::layout::itUnkAttributes_legacy;
#[allow(unused_imports)]
use crate::layout::itWstarAttributes;
#[allow(unused_imports)]
use crate::layout::Collide_LeftWallMask;
#[allow(unused_imports)]
use crate::layout::Collide_RightWallMask;
#[allow(unused_imports)]
use crate::layout::HSD_GObj;
#[allow(unused_imports)]
use crate::layout::HurtCapsule;
#[allow(unused_imports)]
use crate::layout::Item;
#[allow(unused_imports)]
use crate::layout::ItemCommonData;
#[allow(unused_imports)]
use crate::layout::ItemLink;
#[allow(unused_imports)]
use crate::layout::Vec as GcVec;
#[allow(unused_imports)]
use crate::layout::It_Kind_L_Gun_Ray;
#[allow(unused_imports)]
use std::ptr::null_mut as null_mut_itkinds6;
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
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_InitUserData(a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void);
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, isize)>, a2: u8, a3: u32);
    fn HSD_GObjFree(a0: *mut HSD_GObj);
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint);
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) };
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_JObjSetupMatrixSub(a0: *mut HSD_JObj);
    fn HSD_ObjAlloc(a0: *mut _HSD_ObjAllocData) -> *mut c_void => unsafe { melee_runtime::tu::objalloc::HSD_ObjAlloc(a0 as _) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn PSMTXConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void) => unsafe { *(a2 as *mut melee_types::Mtx) = melee_runtime::tu::mtx_pc::PSMTXConcat(&*(a0 as *const _), &*(a1 as *const _)) };
    fn PSMTXCopy(a0: *mut c_void, a1: *mut c_void) => unsafe { *(a1 as *mut melee_types::Mtx) = melee_runtime::tu::mtx_pc::PSMTXCopy(&*(a0 as *const _)) };
    fn PSMTXIdentity(a0: *mut c_void) => unsafe { *(a0 as *mut melee_types::Mtx) = melee_runtime::tu::mtx_pc::PSMTXIdentity() };
    fn ScriptGame_HitRuleCreate(a0: *mut Fighter, a1: *mut HitCapsule, a2: i32);
    fn ScriptGame_HitRuleForget(a0: *mut HitCapsule);
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec, a3: *mut c_void) -> *mut c_void => unsafe { melee_world::tu_ext::efSync_Spawn(a0, a1, a2 as _, a3 as _) };
    fn ftCliffCommon_80081370(a0: *mut HSD_GObj);
    fn ftCo_80090780(a0: *mut HSD_GObj);
    fn ftCo_8009B390(a0: *mut HSD_GObj, a1: f32);
    fn ftCo_800A4A40(a0: *mut Fighter) -> *mut Fighter;
    fn ftCo_800C3A14(a0: *mut HSD_GObj) -> i32;
    fn ftCo_800C3CC0(a0: *mut HSD_GObj);
    fn ftColl_800768A0(a0: *mut Fighter, a1: *mut HitCapsule);
    fn ftColl_8007ABD0(a0: *mut HitCapsule, a1: u32, a2: *mut HSD_GObj);
    fn ftColl_8007AC9C(a0: *mut HitCapsule, a1: i32, a2: *mut HSD_GObj);
    fn ftColl_8007AFF8(a0: *mut HSD_GObj);
    fn ftCommon_8007E2F4(a0: *mut Fighter, a1: i16);
    fn ftLib_GetPartJObj(a0: *mut HSD_GObj, a1: u32) -> *mut HSD_JObj => unsafe { crate::tu::ftlib::ftLib_GetPartJObj(a0, a1) };
    fn ftParts_GetBoneIndex(a0: *mut Fighter, a1: u32) -> u32;
    fn ft_80082E3C(a0: *mut HSD_GObj) -> *mut HSD_GObj;
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_802A2428(a0: *mut HSD_GObj) => unsafe { crate::tu::itlinkhookshot::it_802A2428(a0) };
    fn it_802A2474(a0: *mut c_void) => unsafe { crate::tu::itlinkhookshot::it_802A2474(a0) };
    fn it_802A24A0(a0: *mut HSD_GObj, a1: isize) => unsafe { crate::tu::itlinkhookshot::it_802A24A0(a0, a1) };
    fn it_802A3C98(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> f32 => unsafe { crate::tu::itlinkhookshot::it_802A3C98(a0 as _, a1 as _, a2 as _) };
    fn it_802A3D90(a0: *mut ItemLink) -> i32 => unsafe { crate::tu::itlinkhookshot::it_802A3D90(a0) };
    fn it_802A43B8(a0: *mut ItemLink) => unsafe { crate::itkinds6::it_802a43b8(&mut *a0) };
    fn it_802A43EC(a0: *mut ItemLink) => unsafe { crate::itkinds6::it_802a43ec(&mut *a0) };
    fn it_802A4420(a0: *mut ItemLink) => unsafe { crate::itkinds6::it_802a4420(&mut *a0) };
    fn it_802A4454(a0: *mut ItemLink) => unsafe { crate::tu::itlinkhookshot::it_802A4454(a0) };
    fn it_802A7168(a0: *mut Item, a1: *mut Vec, a2: f32) => unsafe { crate::tu::itlinkhookshot::it_802A7168(a0, a1 as _, a2) };
    fn lbVector_Mirror(a0: *mut Vec, a1: *mut Vec) => unsafe { melee_runtime::tu::lbvector::lbVector_Mirror(a0, a1) };
    fn lbVector_Normalize(a0: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_Normalize(a0) };
    fn mpCheckAllRemap(a0: *mut Vec, a1: *mut i32, a2: *mut u32, a3: *mut Vec, a4: i32, a5: i32, a6: f32, a7: f32, a8: f32, a9: f32) -> i32 => unsafe { melee_world::tu::mplib::mpCheckAllRemap(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9) };
    fn mpColl_80041EE4(a0: *mut CollData) => unsafe { melee_world::tu::mpcoll::mpColl_80041EE4(a0) };
    fn mpColl_SetECBSource_Fixed(a0: *mut CollData, a1: *mut HSD_GObj, a2: f32, a3: f32, a4: f32, a5: f32) => unsafe { melee_world::tu::mpcoll::mpColl_SetECBSource_Fixed(a0, a1, a2, a3, a4, a5) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn it_802A2474(a0: *mut c_void) { super::it_802A2474(a0) }
        pub unsafe extern "C" fn it_802A24A0(a0: *mut HSD_GObj, a1: isize) { super::it_802A24A0(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
    pub use crate::tu::item::item_link_alloc_data;
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

/// `GET_FIGHTER` (ft/inlines.h)
#[inline]
unsafe fn GET_FIGHTER(gobj: *mut HSD_GObj) -> *mut Fighter {
    HSD_GObjGetUserData(gobj) as *mut Fighter
}

/// `HSD_JObjSetupMatrix` (baselib/jobj.h, inline)
#[inline]
unsafe fn HSD_JObjSetupMatrix(jobj: *mut HSD_JObj) {
    if jobj.is_null() || !HSD_JObjMtxIsDirty(jobj) {
        return;
    }
    ext::HSD_JObjSetupMatrixSub(jobj);
}

/// `HSD_JObjCopyMtx` (baselib/jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjCopyMtx(jobj: *mut HSD_JObj, mtx: *mut c_void) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    hsd_assert(!mtx.is_null(), "(mtx)");
    ext::PSMTXCopy(mtx, (*jobj).mtx.as_mut_ptr() as *mut c_void);
}

/// `ItemStateTable it_803F73A8[]`
pub static mut it_803F73A8: [ItemStateTable; 9] = [
    st_row(-1, None, Some(itSamusgrapple_UnkMotion0_Phys), None),
    st_row(-1, None, Some(itSamusgrapple_UnkMotion1_Phys), None),
    st_row(-1, None, Some(itSamusgrapple_UnkMotion2_Phys), None),
    st_row(-1, None, Some(itSamusgrapple_UnkMotion3_Phys), None),
    st_row(-1, None, Some(itSamusgrapple_UnkMotion4_Phys), None),
    st_row(-1, None, Some(itSamusgrapple_UnkMotion5_Phys), None),
    st_row(-1, None, Some(itSamusgrapple_UnkMotion6_Phys), None),
    st_row(-1, None, Some(itSamusgrapple_UnkMotion7_Phys), None),
    st_row(-1, None, Some(itSamusgrapple_UnkMotion8_Phys), None),
];

/// `const itSamusGrapple_Hitbox it_803B8660`: the beam's hitbox command (GameCube bit order, built through
/// the generated accessors).
pub fn it_803B8660() -> itSamusGrapple_Hitbox {
    melee_core::ct!("it_803B8660");
    let mut h: itSamusGrapple_Hitbox = unsafe { core::mem::zeroed() };
    // { 11, 0, 0, 0, 139, 0, 0 }
    h.create_hitbox_0.set_opcode(11);
    h.create_hitbox_0.set_id(0);
    h.create_hitbox_0.set_hit_group(0);
    h.create_hitbox_0.set_only_hit_grabbed(0);
    h.create_hitbox_0.set_bone(139);
    h.create_hitbox_0.set_use_common_bone_ids(0);
    h.create_hitbox_0.set_damage(0);
    // { 1200, 0 }
    h.create_hitbox_1.set_size(1200);
    h.create_hitbox_1.set_z_offset(0);
    // { 0, 0 }
    h.create_hitbox_2.set_y_offset(0);
    h.create_hitbox_2.set_x_offset(0);
    // { 361, 100, 0, 1, 0, 0, 1, 0 }
    h.create_hitbox_3.set_angle(361);
    h.create_hitbox_3.set_knockback_growth(100);
    h.create_hitbox_3.set_weight_set_knockback(0);
    h.create_hitbox_3.set_item_hit_interaction(1);
    h.create_hitbox_3.set_ignore_thrown_fighters(0);
    h.create_hitbox_3.set_ignore_fighter_scale(0);
    h.create_hitbox_3.set_clank(1);
    h.create_hitbox_3.set_rebound(0);
    // { 0, 8, 0, 1, 2, 1, 0 }
    h.create_hitbox_4.set_base_knockback(0);
    h.create_hitbox_4.set_element(8);
    h.create_hitbox_4.set_shield_damage(0);
    h.create_hitbox_4.set_hit_sfx_severity(1);
    h.create_hitbox_4.set_hit_sfx_kind(2);
    h.create_hitbox_4.set_hit_grounded(1);
    h.create_hitbox_4.set_hit_aerial(0);
    h
}

/// `const Vec3 it_803B8674`
pub const it_803B8674: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

/// `samus_grapple_fighter_compare` (static inline)
#[inline]
fn samus_grapple_fighter_compare(id: FtMotionId) -> bool {
    id == 0x165 || id == 0x166 || id == 0xD4 || id == 0xD6
}

/// `Item_InitSpawnPosition` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec3, on_plane: bool) {
    (*spawn).prev_pos = *pos;
    if on_plane {
        (*spawn).prev_pos.z = 0.0;
    }
    (*spawn).pos = (*spawn).prev_pos;
}

/// `samus_grapple_init_link` (static inline)
#[inline]
unsafe fn samus_grapple_init_link(link: *mut ItemLink, jobj: *mut HSD_JObj, gobj: *mut HSD_GObj, pos: *mut Vec3) {
    (*link).jobj = jobj;
    (*link).gobj = gobj;
    (*link).vel = *pos;
    (*link).pos = *pos;
    (*link).set_x2C_b0(0);
    (*link).set_x2C_b1(0);
    (*link).set_x2C_b2(0);
    (*link).coll_data.cur_pos = (*link).pos;
    (*link).coll_data.last_pos = (*link).coll_data.cur_pos;
    ext::mpColl_80041EE4(&mut (*link).coll_data);
}

/// `samus_grapple_calc_grav` (static inline): two `HSD_Randf` draws when the first exceeds 0.9
#[inline]
unsafe fn samus_grapple_calc_grav(vel_y: f32) -> f32 {
    if ext::HSD_Randf() as f64 > 0.9 {
        if (vel_y as f64) < 0.0 {
            -fmadds(0.6, ext::HSD_Randf(), -vel_y)
        } else {
            fmadds(0.6, ext::HSD_Randf(), vel_y)
        }
    } else {
        0.0
    }
}

/// `samus_grapple_setup_pos` (static inline)
#[inline]
unsafe fn samus_grapple_setup_pos(link: *mut ItemLink, pos: *mut Vec3, m: *mut [[f32; 4]; 3]) {
    ext::PSMTXIdentity(m as *mut c_void);
    HSD_JObjSetupMatrix((*link).jobj);
    (*pos).x = (*(*link).jobj).mtx[0][3];
    (*pos).y = (*(*link).jobj).mtx[1][3];
    (*pos).z = (*(*link).jobj).mtx[2][3];
}

/// `samus_grapple_anim` (static inline)
#[inline]
unsafe fn samus_grapple_anim(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::HSD_JObjAnimAll(GET_JOBJ(gobj));
    let mut link = (*ip).xDD4_itemVar.samusgrapple.x4;
    while !link.is_null() {
        ext::HSD_JObjAnimAll((*(*link).gobj).hsd_obj as *mut HSD_JObj);
        link = (*link).prev;
    }
}

/// `samus_grapple_state_sync` (static inline)
#[inline]
unsafe fn samus_grapple_state_sync(fp: *mut Fighter) {
    match (*fp).u.ss.x2240 {
        0 => {
            if (*fp).input.held_buttons[0] & 8 != 0 {
                (*fp).u.ss.x2240 = (*fp).u.ss.x2240.wrapping_add(1);
            }
        }
        1 => {
            if (*fp).input.held_buttons[0] & 4 != 0 {
                (*fp).u.ss.x2240 = (*fp).u.ss.x2240.wrapping_add(1);
            }
        }
        2 => {
            if (*fp).input.held_buttons[0] & 8 != 0 {
                (*fp).u.ss.x2240 = (*fp).u.ss.x2240.wrapping_add(1);
            }
        }
        3 => {
            if (*fp).input.pressed_buttons & 0x100 != 0 {
                (*fp).u.ss.x2240 = (*fp).u.ss.x2240.wrapping_add(1);
            }
        }
        _ => {}
    }
}

pub unsafe extern "C" fn itSamusGrapple_Logic53_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itSamusGrapple_Logic53_Spawned");
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.samusgrapple.unk_10 = None;
}

pub unsafe extern "C" fn it_802B7160(gobj: *mut Fighter_GObj, data: *mut itSamusGrapple_HitboxData) {
    melee_core::ct!("it_802B7160");
    let fp = GET_FIGHTER(gobj);
    let hit_group = (*data).create_hitbox.create_hitbox_0.hit_group();
    let hitbox: *mut HitCapsule = &mut (*fp).x914[(*data).create_hitbox.create_hitbox_0.id() as usize];
    if (*hitbox).state == HitCapsule_Disabled || (*hitbox).x4 != hit_group {
        (*hitbox).x4 = hit_group;
        (*hitbox).state = HitCapsule_Enabled;
        (*fp).set_x2219_b3(1);
        ext::ftColl_800768A0(fp, hitbox);
    }
    {
        let mut bone = (*data).create_hitbox.create_hitbox_0.bone();
        if (*data).create_hitbox.create_hitbox_0.use_common_bone_ids() != 0 {
            bone = ext::ftParts_GetBoneIndex(fp, bone as _) as u32;
            (*hitbox).jobj = (*(*fp).parts.add(bone as usize)).joint;
        } else {
            (*hitbox).jobj = (*(*fp).parts.add(bone as usize)).joint;
        }
    }
    // damage_arg = *(u16*)((u8*)data + 2) & 0x3FF, the packed `damage` field on the GameCube
    let damage_arg = (*data).create_hitbox.create_hitbox_0.damage();
    ext::ScriptGame_HitRuleForget(hitbox);
    ext::ftColl_8007ABD0(hitbox, damage_arg, gobj);
    (*hitbox).scale = ((*data).create_hitbox.create_hitbox_1.size() as f32) * 0.003906f32;
    (*hitbox).b_offset.x = ((*data).create_hitbox.create_hitbox_1.z_offset() as f32) * 0.003906f32;
    (*hitbox).b_offset.y = ((*data).create_hitbox.create_hitbox_2.y_offset() as f32) * 0.003906f32;
    (*hitbox).b_offset.z = ((*data).create_hitbox.create_hitbox_2.x_offset() as f32) * 0.003906f32;
    ext::ftColl_8007AC9C(hitbox, (*data).create_hitbox.create_hitbox_3.angle() as i32, gobj);
    (*hitbox).x24 = (*data).create_hitbox.create_hitbox_3.knockback_growth() as _;
    (*hitbox).x28 = (*data).create_hitbox.create_hitbox_3.weight_set_knockback() as _;
    (*hitbox).set_x43_b0((*data).create_hitbox.create_hitbox_3.item_hit_interaction() as _);
    (*hitbox).set_x43_b1((*data).create_hitbox.create_hitbox_3.ignore_fighter_scale() as _);
    (*hitbox).set_x40_b0((*data).create_hitbox.create_hitbox_3.clank() as _);
    (*hitbox).set_x40_b1((*data).create_hitbox.create_hitbox_3.rebound() as _);
    (*hitbox).x2C = (*data).create_hitbox.create_hitbox_4.base_knockback() as _;
    (*hitbox).element = (*data).create_hitbox.create_hitbox_4.element() as _;
    (*hitbox).x34 = (*data).create_hitbox.create_hitbox_4.shield_damage() as _;
    (*hitbox).sfx_severity = (*data).create_hitbox.create_hitbox_4.hit_sfx_severity() as _;
    (*hitbox).sfx_kind = (*data).create_hitbox.create_hitbox_4.hit_sfx_kind() as _;
    (*hitbox).set_x40_b2((*data).create_hitbox.create_hitbox_4.hit_aerial() as _);
    // hitbox_flags->hit_grounded (byte 3 of create_hitbox_4, bit 6): the `hit_grounded` field
    (*hitbox).set_x40_b3((*data).create_hitbox.create_hitbox_4.hit_grounded() as _);
    (*hitbox).set_x42_b5(1);
    (*hitbox).set_x42_b7(1);
    (*hitbox).set_x41_b4(0);
    (*hitbox).set_x41_b6(0);
    (*hitbox).set_x41_b5(0);
    (*hitbox).set_x42_b0(0);
    (*hitbox).set_x42_b4(0);
    (*hitbox).set_x41_b7(0);
    (*hitbox).x134.set_hit_grabbed_victim_only((*data).x134_4() as _);
    (*hitbox).set_x42_b1(1);
    (*hitbox).set_x42_b2(0);
    (*hitbox).set_x43_b2(0);
    ext::ScriptGame_HitRuleCreate(fp, hitbox, 0);
}

pub unsafe extern "C" fn it_802B743C(gobj: *mut HSD_GObj, ip: *mut Item, type_: i32) {
    melee_core::ct!("it_802B743C");
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusGrappleAttributes;
    let jobj: *mut HSD_JObj;
    let mut anim: *mut HSD_AnimJoint;
    let mut matanim: *mut HSD_MatAnimJoint;
    let mut shapeanim: *mut HSD_ShapeAnimJoint;

    if type_ == 0 {
        jobj = ext::HSD_JObjLoadJoint((*attrs).x64);
    } else if type_ == 1 {
        jobj = ext::HSD_JObjLoadJoint((*attrs).x68);
    } else {
        jobj = ext::HSD_JObjLoadJoint((*attrs).x6C);
    }
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::cb::it_802A24A0), 6, 0);
    if type_ == 0 {
        shapeanim = if !(*attrs).x7C.is_null() { *(*attrs).x7C } else { null_mut() };
        matanim = if !(*attrs).x78.is_null() { *(*attrs).x78 } else { null_mut() };
        anim = if !(*attrs).x74.is_null() { *(*attrs).x74 } else { null_mut() };
        ext::HSD_JObjAddAnimAll((*jobj).child, anim, matanim, shapeanim);
    } else if type_ == 1 {
        shapeanim = if !(*attrs).x88.is_null() { *(*attrs).x88 } else { null_mut() };
        matanim = if !(*attrs).x84.is_null() { *(*attrs).x84 } else { null_mut() };
        anim = if !(*attrs).x80.is_null() { *(*attrs).x80 } else { null_mut() };
        ext::HSD_JObjAddAnimAll((*jobj).child, anim, matanim, shapeanim);
    } else {
        shapeanim = if !(*attrs).x94.is_null() { *(*attrs).x94 } else { null_mut() };
        matanim = if !(*attrs).x90.is_null() { *(*attrs).x90 } else { null_mut() };
        anim = if !(*attrs).x8C.is_null() { *(*attrs).x8C } else { null_mut() };
        ext::HSD_JObjAddAnimAll((*jobj).child, anim, matanim, shapeanim);
    }
    ext::HSD_JObjReqAnimAll((*jobj).child, 0.0);
}

/// `samus_grapple_setup_tail` (static inline)
#[inline]
unsafe fn samus_grapple_setup_tail(ip: *mut Item, link_gobj: *mut HSD_GObj) {
    let attrs2 = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusGrappleAttributes;
    let tail_jobj = ext::HSD_JObjLoadJoint((*attrs2).x70);
    ext::HSD_GObjObject_80390A70(link_gobj, ext::HSD_GObj_JObjKind, tail_jobj as *mut c_void);
    ext::GObj_SetupGXLink(link_gobj, Some(ext::cb::it_802A24A0), 6, 0);
    // the arguments are evaluated in order: anim, matanim, shapeanim
    let a = if !(*attrs2).x98.is_null() { *(*attrs2).x98 } else { null_mut() };
    let b = if !(*attrs2).x9C.is_null() { *(*attrs2).x9C } else { null_mut() };
    let c = if !(*attrs2).xA0.is_null() { *(*attrs2).xA0 } else { null_mut() };
    ext::HSD_JObjAddAnimAll((*tail_jobj).child, a, b, c);
    ext::HSD_JObjReqAnimAll((*tail_jobj).child, 0.0);
}

pub unsafe extern "C" fn it_802B75FC(ip: *mut Item, jobj_arg: *mut HSD_JObj, arg2: i32, scale: f32) -> *mut HSD_JObj {
    melee_core::ct!("it_802B75FC");
    let coeff: f32;
    let mut prev_link: *mut ItemLink;
    let attrs: *mut itSamusGrappleAttributes;
    let mut head_link: *mut ItemLink = null_mut();
    let mut tail_link: *mut ItemLink = null_mut();
    let mut result: *mut HSD_JObj = null_mut();
    let mut zero_vel: Vec3;

    attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusGrappleAttributes;
    zero_vel = it_803B8674;

    if arg2 != 0 {
        coeff = (*attrs).x60;
    } else {
        coeff = (*attrs).x5C;
    }

    (*attrs).x38 = (*attrs).x10 * scale;
    (*attrs).x3C = (*attrs).x14 * scale;
    (*attrs).x40 = coeff * ((*attrs).x18 * scale);
    (*attrs).x44 = (*attrs).x1C * scale;
    (*attrs).x48 = coeff * ((*attrs).x20 * scale);
    (*attrs).x4C = coeff * ((*attrs).x24 * scale);
    (*attrs).x50 = coeff * ((*attrs).x28 * scale);
    (*attrs).x54 = coeff * ((*attrs).x2C * scale);
    (*attrs).x58 = (*attrs).x30 * scale;

    let temp: f32;
    if (scale as f64) > 1.0 {
        // (x8 * (xC * x38)) + ((1.0f - x8) * (xC * x10)): contracted around the left product
        temp = fmadds(
            (*attrs).x8,
            ((*attrs).xC as f32) * (*attrs).x38,
            (1.0f32 - (*attrs).x8) * (((*attrs).xC as f32) * (*attrs).x10),
        );
    } else {
        temp = ((*attrs).xC as f32) * (*attrs).x38;
    }
    (*attrs).x34 = melee_core::ppc::fctiwz_f32((temp * coeff) / (*attrs).x38);

    if (*ip).xDD4_itemVar.samusgrapple.x16 == 1 {
        (*attrs).x34 = (*attrs).x34.wrapping_mul(2);
    }
    (*ip).xDD4_itemVar.samusgrapple.x15 = (*attrs).x34 as u8;

    prev_link = null_mut();
    let mut i: i32 = 0;
    while i < (*attrs).x34 {
        let gobj_tmp = ext::GObj_Create(7, 0xA, 0);
        let link_gobj = gobj_tmp;
        if link_gobj.is_null() {
            while !prev_link.is_null() {
                ext::HSD_GObjFree((*prev_link).gobj);
                prev_link = (*prev_link).next;
            }
            return null_mut();
        }
        let link_tmp = ext::HSD_ObjAlloc(&raw mut ext::item_link_alloc_data) as *mut ItemLink;
        let link = link_tmp;
        ext::GObj_InitUserData(link_gobj, 6, Some(ext::cb::it_802A2474), link as *mut c_void);

        if i == 0 {
            (*link).next = null_mut();
            let first_coll = &mut (*link).coll_data as *mut CollData;
            head_link = link;
            samus_grapple_init_link(link, jobj_arg, link_gobj, &mut zero_vel);
            ext::mpColl_SetECBSource_Fixed(first_coll, null_mut(), 1.0, 1.0, 1.0, 1.0);
            it_802B743C(link_gobj, ip, 0);
        } else if i == (*attrs).x34 - 1 {
            (*prev_link).prev = link;
            let last_coll = &mut (*link).coll_data as *mut CollData;
            (*link).prev = null_mut();
            tail_link = link;
            (*link).next = prev_link;
            samus_grapple_init_link(link, jobj_arg, link_gobj, &mut zero_vel);
            ext::mpColl_SetECBSource_Fixed(last_coll, null_mut(), 1.5, 1.5, 1.5, 1.5);
            samus_grapple_setup_tail(ip, link_gobj);
            result = (*link_gobj).hsd_obj as *mut HSD_JObj;
        } else {
            (*prev_link).prev = link;
            let link_coll = &mut (*link).coll_data as *mut CollData;
            (*link).next = prev_link;
            samus_grapple_init_link(link, jobj_arg, link_gobj, &mut zero_vel);
            ext::mpColl_SetECBSource_Fixed(link_coll, null_mut(), 1.0, 1.0, 1.0, 1.0);
            it_802B743C(link_gobj, ip, i % 3);
        }
        (*link).x1CC = -1;
        prev_link = link;
        i += 1;
    }

    (*ip).xDD4_itemVar.samusgrapple.x0 = tail_link;
    (*ip).xDD4_itemVar.samusgrapple.x4 = head_link;
    result
}

pub unsafe extern "C" fn it_802B7B84(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802B7B84");
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        if !ip.is_null() {
            if !(*ip).xDD4_itemVar.samusgrapple.x8.is_null() {
                let fp = GET_FIGHTER((*ip).xDD4_itemVar.samusgrapple.x8);
                if !fp.is_null() {
                    (*ip).xDD4_itemVar.samusgrapple.unk_10 = None;
                    (*fp).u.ss.x223C = null_mut();
                    (*fp).accessory2_cb = None;
                    (*fp).death1_cb = None;
                    (*fp).accessory3_cb = None;
                    {
                        let mut cur = (*ip).xDD4_itemVar.samusgrapple.x0;
                        while !cur.is_null() {
                            let link_gobj = (*cur).gobj;
                            cur = (*cur).next;
                            ext::HSD_GObjFree(link_gobj);
                        }
                        ext::Item_8026A8EC(gobj);
                    }
                }
            }
        }
    }
}

pub unsafe extern "C" fn it_802B7C18(owner: *mut Fighter_GObj, pos: *mut Vec3, facing_dir: f32) -> *mut Item_GObj {
    melee_core::ct!("it_802B7C18");
    let fp = GET_FIGHTER(owner);
    let mut spawn: SpawnItem = core::mem::zeroed();

    if !samus_grapple_fighter_compare((*fp).motion_id) {
        return null_mut();
    }

    spawn.kind = It_Kind_Samus_GBeam;
    Item_InitSpawnPosition(&mut spawn, pos, false);
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = owner;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.x0.set_b0(1);

    let item_gobj = ext::Item_80268B18(&mut spawn);
    if !item_gobj.is_null() {
        let ip = GET_ITEM(item_gobj);
        let jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
        let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusGrappleAttributes;
        (*ip).xDD4_itemVar.samusgrapple.x14 = 0;

        if (*fp).u.ss.x2240 >= 4 {
            if (*fp).ground_or_air == GA_Ground {
                (*ip).xDD4_itemVar.samusgrapple.x16 = 1;
            } else {
                (*ip).xDD4_itemVar.samusgrapple.x16 = 2;
            }
        } else {
            (*ip).xDD4_itemVar.samusgrapple.x16 = 0;
            (*fp).u.ss.x2240 = 0;
        }

        let type_: i32 = if (*fp).motion_id == 0x165 { 1 } else { 0 };
        if it_802B75FC(ip, ext::ftLib_GetPartJObj(owner, FtPart_ThrowN), type_, (*fp).x34_scale.y).is_null() {
            ext::Item_8026A8EC(item_gobj);
            return null_mut();
        }

        let shapeanim = if !(*attrs).xAC.is_null() { *(*attrs).xAC } else { null_mut() };
        let matanim = if !(*attrs).xA8.is_null() { *(*attrs).xA8 } else { null_mut() };
        let anim = if !(*attrs).xA4.is_null() { *(*attrs).xA4 } else { null_mut() };
        ext::HSD_JObjAddAnimAll((*jobj).child, anim, matanim, shapeanim);
        ext::HSD_JObjReqAnimAll((*jobj).child, 0.0);

        (*ip).xDD4_itemVar.samusgrapple.x8 = owner;
        (*ip).xDD4_itemVar.samusgrapple.xC = (*(*(*ip).xDD4_itemVar.samusgrapple.x0).gobj).hsd_obj as *mut HSD_JObj;
        (*(*fp).parts.add(0x8B)).joint = (*ip).xDD4_itemVar.samusgrapple.xC;
        ext::Item_8026AB54(item_gobj, owner, FtPart_ThrowN);
        ext::it_802A2428(item_gobj);
    }
    item_gobj
}

/// `fn_802B7E34_inline` (static inline)
#[inline]
unsafe fn fn_802B7E34_inline(gobj: *mut Item_GObj, ip: *mut Item, m: *mut [[f32; 4]; 3]) -> *mut Item {
    let link = (*ip).xDD4_itemVar.samusgrapple.x0;
    let jobj = (*(*link).gobj).hsd_obj as *mut HSD_JObj;
    HSD_JObjSetupMatrix((*link).jobj);
    ext::PSMTXIdentity(m as *mut c_void);
    ext::PSMTXConcat((*(*link).jobj).mtx.as_mut_ptr() as *mut c_void, m as *mut c_void, m as *mut c_void);
    HSD_JObjCopyMtx(jobj, m as *mut c_void);
    (*jobj).flags |= 0x03800000;
    HSD_JObjSetMtxDirty(jobj);
    GET_ITEM(gobj)
}

/// `fn_802B7E34_anim_inline` (static inline)
#[inline]
unsafe fn fn_802B7E34_anim_inline(gobj: *mut Item_GObj, ip: *mut Item, m: *mut [[f32; 4]; 3]) {
    {
        let link = (*ip).xDD4_itemVar.samusgrapple.x0;
        let jobj = (*(*link).gobj).hsd_obj as *mut HSD_JObj;
        HSD_JObjSetupMatrix((*link).jobj);
        ext::PSMTXIdentity(m as *mut c_void);
        ext::PSMTXConcat((*(*link).jobj).mtx.as_mut_ptr() as *mut c_void, m as *mut c_void, m as *mut c_void);
        HSD_JObjCopyMtx(jobj, m as *mut c_void);
        (*jobj).flags |= 0x03800000;
        HSD_JObjSetMtxDirty(jobj);
    }
    {
        let ip = GET_ITEM(gobj);
        ext::HSD_JObjAnimAll(GET_JOBJ(gobj));
        let mut link = (*ip).xDD4_itemVar.samusgrapple.x4;
        while !link.is_null() {
            ext::HSD_JObjAnimAll((*(*link).gobj).hsd_obj as *mut HSD_JObj);
            link = (*link).prev;
        }
    }
}

pub unsafe extern "C" fn fn_802B7E34(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_802B7E34");
    let mut m: [[f32; 4]; 3] = core::mem::zeroed();
    let ip = GET_ITEM(gobj);
    let fp = GET_FIGHTER((*ip).owner);

    samus_grapple_state_sync(fp);
    fn_802B7E34_anim_inline(gobj, ip, &mut m);
    if samus_grapple_fighter_compare((*fp).motion_id) {
        return;
    }
    it_802B7B84(gobj);
}

pub unsafe extern "C" fn itSamusgrapple_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSamusgrapple_UnkMotion0_Phys");
    (*GET_ITEM(gobj)).xDD4_itemVar.samusgrapple.unk_10 = Some(fn_802B7E34);
}

pub unsafe extern "C" fn fn_802B805C(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_802B805C");
    let ip = GET_ITEM(gobj);
    let mut pos: Vec3 = core::mem::zeroed();
    let mut normal: Vec3;
    let mut pos2: Vec3;
    let mut m: [[f32; 4]; 3] = core::mem::zeroed();

    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusGrappleAttributes;
    let fp = (*(*ip).owner).user_data as *mut Fighter;
    samus_grapple_state_sync(fp);

    let link = (*ip).xDD4_itemVar.samusgrapple.x0;
    if !samus_grapple_fighter_compare((*fp).motion_id) && (*(*link).next).x2C_b0() == 0 {
        it_802B7B84(gobj);
        return;
    }

    samus_grapple_setup_pos(link, &mut pos, &mut m);

    match it_802B9328(link, &mut pos, attrs, (*(*ip).owner).user_data as *mut Fighter) {
        1 => {
            if (*fp).motion_id == 0x165 {
                ext::ftCo_800C3CC0((*ip).owner);
                it_802BAB40(gobj);
                {
                    let link2 = (*ip).xDD4_itemVar.samusgrapple.x0;
                    pos2 = (*link2).pos;
                    {
                        let link_gobj = (*(*ip).xDD4_itemVar.samusgrapple.x0).gobj;
                        ext::efSync_Spawn(0x41C, link_gobj, &mut pos2, link2 as *mut c_void);
                        ext::efSync_Spawn(0x3F1, link_gobj, &mut pos2, &mut (*fp).facing_dir as *mut f32 as *mut c_void);
                    }
                }
                return;
            }
            (*link).vel.x *= -(*attrs).x0;
            it_802BAA08(gobj);
            ext::ftCommon_8007E2F4(fp, 0);
        }
        2 => {
            normal = (*link).coll_data.floor.normal;
            ext::lbVector_Normalize(&mut normal);
            ext::lbVector_Mirror(&mut (*link).vel, &mut normal);
            (*link).vel.y *= (*attrs).x0;
            it_802BAA08(gobj);
            ext::ftCommon_8007E2F4(fp, 0);
        }
        3 => {
            it_802BA9B8(gobj);
            ext::ftCommon_8007E2F4(fp, 0);
        }
        _ => {}
    }
    ext::it_802A7168(ip, &mut pos, (*fp).x34_scale.y);
    samus_grapple_anim(gobj);
}

pub unsafe extern "C" fn itSamusgrapple_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSamusgrapple_UnkMotion1_Phys");
    (*GET_ITEM(gobj)).xDD4_itemVar.samusgrapple.unk_10 = Some(fn_802B805C);
}

pub unsafe extern "C" fn fn_802B8384(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_802B8384");
    let ip = GET_ITEM(gobj);
    let owner = (*ip).owner;
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusGrappleAttributes;
    let mut pos: Vec3 = core::mem::zeroed();
    let mut sp5C: Vec3;
    let mut m: [[f32; 4]; 3] = core::mem::zeroed();

    let fp = (*owner).user_data as *mut Fighter;
    let link = (*ip).xDD4_itemVar.samusgrapple.x0;
    samus_grapple_setup_pos(link, &mut pos, &mut m);

    match it_802B99A0(link, &mut pos, attrs, fp) {
        1 => {
            (*link).vel.x *= -(*attrs).x0;
        }
        2 => {
            sp5C = (*link).coll_data.floor.normal;
            ext::lbVector_Normalize(&mut sp5C);
            ext::lbVector_Mirror(&mut (*link).vel, &mut sp5C);
            (*link).vel.y *= (*attrs).x0;
        }
        3 => {
            it_802BA9B8(gobj);
        }
        _ => {}
    }
    ext::it_802A7168(ip, &mut pos, (*fp).x34_scale.y);
    samus_grapple_anim(gobj);
}

pub unsafe extern "C" fn itSamusgrapple_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSamusgrapple_UnkMotion2_Phys");
    (*GET_ITEM(gobj)).xDD4_itemVar.samusgrapple.unk_10 = Some(fn_802B8384);
}

pub unsafe extern "C" fn fn_802B8524(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_802B8524");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusGrappleAttributes;
    let fp = (*(*ip).owner).user_data as *mut Fighter;
    let fp2 = fp;
    let link = (*ip).xDD4_itemVar.samusgrapple.x4;
    let mut pos: Vec3 = core::mem::zeroed();
    let mut m: [[f32; 4]; 3] = core::mem::zeroed();

    samus_grapple_setup_pos(link, &mut pos, &mut m);
    it_802B9CE8(link, &mut pos, attrs, (*(*ip).owner).user_data as *mut Fighter);
    ext::it_802A7168(ip, &mut pos, (*fp2).x34_scale.y);
    samus_grapple_anim(gobj);
    if samus_grapple_fighter_compare((*fp).motion_id) {
        return;
    }
    it_802BAA58(gobj);
}

pub unsafe extern "C" fn itSamusgrapple_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSamusgrapple_UnkMotion3_Phys");
    (*GET_ITEM(gobj)).xDD4_itemVar.samusgrapple.unk_10 = Some(fn_802B8524);
}

pub unsafe extern "C" fn fn_802B8684(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_802B8684");
    let mut pos: Vec3 = core::mem::zeroed();
    let mut m: [[f32; 4]; 3] = core::mem::zeroed();

    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusGrappleAttributes;
    let fp = (*(*ip).owner).user_data as *mut Fighter;
    let fp2 = fp;
    let link = (*ip).xDD4_itemVar.samusgrapple.x4;
    samus_grapple_setup_pos(link, &mut pos, &mut m);

    let retract_speed: f32 = if (*fp2).ground_or_air == GA_Ground {
        (*attrs).x4C
    } else {
        (2.0f64 * ((*attrs).x4C as f64)) as f32
    };
    if it_802BA194(link, &mut pos, attrs, retract_speed) {
        if !samus_grapple_fighter_compare((*fp2).motion_id) {
            it_802B7B84(gobj);
            return;
        }
        itSamusGrapple_Logic53_PickedUp(gobj);
    }
    ext::it_802A7168(ip, &mut pos, (*fp2).x34_scale.y);
    samus_grapple_anim(gobj);
}

pub unsafe extern "C" fn itSamusgrapple_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSamusgrapple_UnkMotion4_Phys");
    (*GET_ITEM(gobj)).xDD4_itemVar.samusgrapple.unk_10 = Some(fn_802B8684);
}

pub unsafe extern "C" fn fn_802B8814(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_802B8814");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusGrappleAttributes;
    let mut pos: Vec3 = core::mem::zeroed();
    let mut m: [[f32; 4]; 3] = core::mem::zeroed();
    let fp = (*(*ip).owner).user_data as *mut Fighter;

    if (*ip).xDD4_itemVar.samusgrapple.x14 != 0 {
        it_802B7B84(gobj);
        return;
    }

    let link = (*ip).xDD4_itemVar.samusgrapple.x4;
    samus_grapple_setup_pos(link, &mut pos, &mut m);

    if it_802BA2D8(link, &mut pos, attrs, (*attrs).x54) {
        (*ip).xDD4_itemVar.samusgrapple.x14 = 1;
        return;
    }
    ext::it_802A7168(ip, &mut pos, (*fp).x34_scale.y);
    samus_grapple_anim(gobj);
}

pub unsafe extern "C" fn itSamusgrapple_UnkMotion5_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSamusgrapple_UnkMotion5_Phys");
    (*GET_ITEM(gobj)).xDD4_itemVar.samusgrapple.unk_10 = Some(fn_802B8814);
}

/// `itGrappleCheckCollision` (it/inlines.h, static inline)
#[inline]
unsafe fn itGrappleCheckCollision(head: *mut ItemLink, fp: *mut Fighter, pos: *mut Vec3) -> bool {
    let head_pos: *mut Vec3 = &mut (*head).pos;

    if ext::mpCheckAllRemap(null_mut(), null_mut(), null_mut(), null_mut(), -1, -1, (*pos).x, (*pos).y, (*head).pos.x, (*head).pos.y) != 0 {
        true
    } else if ext::mpCheckAllRemap(null_mut(), null_mut(), null_mut(), null_mut(), -1, -1, (*fp).cur_pos.x, (*fp).cur_pos.y, (*head_pos).x, (*head_pos).y) != 0 {
        true
    } else {
        false
    }
}

pub unsafe extern "C" fn fn_802B895C(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_802B895C");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusGrappleAttributes;
    let fp = (*(*ip).owner).user_data as *mut Fighter;
    let link = (*ip).xDD4_itemVar.samusgrapple.x4;
    let mut pos: Vec3 = core::mem::zeroed();
    let mut m: [[f32; 4]; 3] = core::mem::zeroed();

    let fp2 = fp;
    samus_grapple_setup_pos(link, &mut pos, &mut m);

    if itGrappleCheckCollision((*ip).xDD4_itemVar.samusgrapple.x0, fp2, &mut pos) {
        ext::ftCo_80090780((*ip).owner);
        it_802B7B84(gobj);
        return;
    }
    if it_802B9FD4((*ip).xDD4_itemVar.samusgrapple.x0, &mut pos, attrs) {
        it_802BABB8(gobj);
        return;
    }
    ext::it_802A7168(ip, &mut pos, (*fp2).x34_scale.y);
    samus_grapple_anim(gobj);
    if (*fp2).ground_or_air != GA_Air {
        it_802BAA58(gobj);
        return;
    }
    if (*fp2).input.pressed_buttons & 0x100 != 0 {
        it_802BAB7C(gobj);
    }
}

pub unsafe extern "C" fn itSamusgrapple_UnkMotion6_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSamusgrapple_UnkMotion6_Phys");
    (*GET_ITEM(gobj)).xDD4_itemVar.samusgrapple.unk_10 = Some(fn_802B895C);
}

pub unsafe extern "C" fn fn_802B8B54(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_802B8B54");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusGrappleAttributes;
    let fp = (*(*ip).owner).user_data as *mut Fighter;
    let mut pos: Vec3 = core::mem::zeroed();
    let mut m: [[f32; 4]; 3] = core::mem::zeroed();

    let da = (*fp).dat_attrs as *mut ftSs_DatAttrs;
    let link = (*ip).xDD4_itemVar.samusgrapple.x4;
    let fp2 = fp;
    samus_grapple_setup_pos(link, &mut pos, &mut m);
    let dx = (*fp2).cur_pos.x - pos.x;
    let dy = (*fp2).cur_pos.y - pos.y;

    if it_802BA3BC((*ip).xDD4_itemVar.samusgrapple.x4, (*ip).xDD4_itemVar.samusgrapple.x0, &mut pos, attrs, (*attrs).x4C) {
        (*fp2).cur_pos.x = pos.x + dx;
        (*fp2).cur_pos.y = pos.y + dy;
        if ext::ftCo_800C3A14((*ip).owner) != 0 && ext::ft_80082E3C((*ip).owner).is_null() {
            ext::ftCliffCommon_80081370((*ip).owner);
        } else {
            (*fp2).self_vel.x = 0.0;
            ext::ftCo_8009B390((*ip).owner, (*da).xCC);
        }
        it_802B7B84(gobj);
        return;
    }
    (*fp2).cur_pos.x = pos.x + dx;
    (*fp2).cur_pos.y = pos.y + dy;
    ext::it_802A7168(ip, &mut pos, (*fp2).x34_scale.y);
    samus_grapple_anim(gobj);
    if (*fp2).ground_or_air != GA_Air {
        it_802BAA58(gobj);
    }
}

pub unsafe extern "C" fn itSamusgrapple_UnkMotion7_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSamusgrapple_UnkMotion7_Phys");
    (*GET_ITEM(gobj)).xDD4_itemVar.samusgrapple.unk_10 = Some(fn_802B8B54);
}

pub unsafe extern "C" fn fn_802B8D38(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_802B8D38");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusGrappleAttributes;
    let fp = (*(*ip).owner).user_data as *mut Fighter;
    let link = (*ip).xDD4_itemVar.samusgrapple.x4;
    let mut pos: Vec3 = core::mem::zeroed();
    let saved_pos: Vec3;
    let mut m: [[f32; 4]; 3] = core::mem::zeroed();

    let fp2 = fp;
    samus_grapple_setup_pos(link, &mut pos, &mut m);

    if itGrappleCheckCollision((*ip).xDD4_itemVar.samusgrapple.x0, fp2, &mut pos) {
        ext::ftCo_80090780((*ip).owner);
        it_802B7B84(gobj);
        return;
    }

    let old_x = (*fp2).cur_pos.x - pos.x;
    let old_y = (*fp2).cur_pos.y - pos.y;
    saved_pos = (*fp2).cur_pos;
    it_802BA5DC((*ip).xDD4_itemVar.samusgrapple.x4, (*ip).xDD4_itemVar.samusgrapple.x0, &mut pos, attrs);
    (*fp2).cur_pos.x = pos.x + old_x;
    (*fp2).cur_pos.y = pos.y + old_y;
    {
        let mut tmp: f32;
        tmp = (*fp2).self_vel.x + ((*fp2).cur_pos.x - saved_pos.x);
        (*fp2).self_vel.x = tmp;
        (*fp2).pos_delta.x = tmp;
        tmp = (*fp2).self_vel.y + ((*fp2).cur_pos.y - saved_pos.y);
        (*fp2).self_vel.y = tmp;
        (*fp2).pos_delta.y = tmp;
    }
    ext::it_802A7168(ip, &mut pos, (*fp2).x34_scale.y);
    samus_grapple_anim(gobj);
    if (*ip).xDD4_itemVar.samusgrapple.x16 != 0 && ((*fp2).input.held_buttons[0] & 0x40) != 0 {
        (*fp2).mv.ss.grapple.x4 += 1.0;
    }
    if (*fp2).ground_or_air != GA_Air {
        it_802BAA58(gobj);
        return;
    }
    if (*fp2).input.pressed_buttons & 0x100 != 0 {
        it_802BAB7C(gobj);
        return;
    }
    let old = (*fp2).mv.ss.grapple.x4;
    (*fp2).mv.ss.grapple.x4 = old - 1.0;
    if old <= 0.0 {
        ext::ftCo_80090780((*ip).owner);
        it_802B7B84(gobj);
    }
}

pub unsafe extern "C" fn itSamusgrapple_UnkMotion8_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSamusgrapple_UnkMotion8_Phys");
    (*GET_ITEM(gobj)).xDD4_itemVar.samusgrapple.unk_10 = Some(fn_802B8D38);
}

pub unsafe extern "C" fn it_802B900C(mut link: *mut ItemLink, pos: *mut Vec3, attrs: *mut itSamusGrappleAttributes, dist: f32) {
    melee_core::ct!("it_802B900C");
    let mut dir: Vec3 = core::mem::zeroed();
    let mut prev = (*link).prev;
    let mut d: f32;

    ext::it_802A3C98(&mut (*link).pos, pos, &mut dir);
    (*link).pos.x = fmadds(dir.x, dist, (*pos).x);
    (*link).pos.y = fmadds(dir.y, dist, (*pos).y);
    (*link).pos.z = fmadds(dir.z, dist, (*pos).z);

    while !prev.is_null() {
        (*prev).vel.y -= samus_grapple_calc_grav((*prev).vel.y);
        ext::it_802A4420(prev);
        ext::it_802A43EC(prev);

        d = ext::it_802A3C98(&mut (*prev).pos, &mut (*link).pos, &mut dir);
        if d > (*attrs).x38 {
            (*prev).pos.x = fmadds(dir.x, (*attrs).x38, (*link).pos.x);
            (*prev).pos.y = fmadds(dir.y, (*attrs).x38, (*link).pos.y);
            (*prev).pos.z = fmadds(dir.z, (*attrs).x38, (*link).pos.z);
        } else if d < (*attrs).x3C {
            (*prev).pos.x = fmadds(dir.x, (*attrs).x3C, (*link).pos.x);
            (*prev).pos.y = fmadds(dir.y, (*attrs).x3C, (*link).pos.y);
            (*prev).pos.z = fmadds(dir.z, (*attrs).x3C, (*link).pos.z);
        }

        link = prev;
        prev = (*link).prev;
    }
}

pub unsafe extern "C" fn it_802B91C4(mut link: *mut ItemLink, pos: *mut Vec3, attrs: *mut itSamusGrappleAttributes, dist: f32) {
    melee_core::ct!("it_802B91C4");
    let mut dir: Vec3 = core::mem::zeroed();
    let mut prev = (*link).prev;
    let mut d: f32;

    ext::it_802A3C98(&mut (*link).pos, pos, &mut dir);
    (*link).pos.x = fmadds(dir.x, dist, (*pos).x);
    (*link).pos.y = fmadds(dir.y, dist, (*pos).y);
    (*link).pos.z = fmadds(dir.z, dist, (*pos).z);

    while !prev.is_null() {
        (*prev).vel.y -= (*attrs).x44;
        ext::it_802A4420(prev);
        ext::it_802A43EC(prev);

        d = ext::it_802A3C98(&mut (*prev).pos, &mut (*link).pos, &mut dir);
        if d > (*attrs).x38 {
            (*prev).pos.x = fmadds(dir.x, (*attrs).x38, (*link).pos.x);
            (*prev).pos.y = fmadds(dir.y, (*attrs).x38, (*link).pos.y);
            (*prev).pos.z = fmadds(dir.z, (*attrs).x38, (*link).pos.z);
        } else if d < (*attrs).x3C {
            (*prev).pos.x = fmadds(dir.x, (*attrs).x3C, (*link).pos.x);
            (*prev).pos.y = fmadds(dir.y, (*attrs).x3C, (*link).pos.y);
            (*prev).pos.z = fmadds(dir.z, (*attrs).x3C, (*link).pos.z);
        }

        link = prev;
        prev = (*prev).prev;
    }
}

/// `it_802B9328_attach` (static inline)
#[inline]
unsafe fn it_802B9328_attach(link: *mut ItemLink, pos: *mut Vec3, m: *mut [[f32; 4]; 3]) {
    ext::PSMTXIdentity(m as *mut c_void);
    HSD_JObjSetupMatrix((*link).jobj);
    (*pos).x = (*(*link).jobj).mtx[0][3];
    (*pos).y = (*(*link).jobj).mtx[1][3];
    (*pos).z = (*(*link).jobj).mtx[2][3];
    (*link).pos = *pos;
    (*link).coll_data.cur_pos = (*link).pos;
    (*link).coll_data.last_pos = (*link).coll_data.cur_pos;
    (*link).set_x2C_b0(1);
}

/// `it_802B9328_grav` (static inline): `one` is a local variable, so the product is contracted
#[inline]
unsafe fn it_802B9328_grav(vely: f32) -> f32 {
    let one: f32 = 1.0;

    if ext::HSD_Randf() as f64 > 0.9 {
        if (vely as f64) < 0.0 {
            -fmadds(one, ext::HSD_Randf(), -vely)
        } else {
            fmadds(one, ext::HSD_Randf(), vely)
        }
    } else {
        0.0
    }
}

pub unsafe extern "C" fn it_802B9328(link: *mut ItemLink, pos: *mut Vec3, attrs: *mut itSamusGrappleAttributes, fp: *mut Fighter) -> i32 {
    melee_core::ct!("it_802B9328");
    let mut cur: *mut ItemLink;
    let mut next: *mut ItemLink;
    let grapple_ip = (*(*fp).u.ss.x223C).user_data as *mut Item;
    let da = (*(*fp).ft_data).ext_attr as *mut ftSs_DatAttrs;
    let mut dir: Vec3 = core::mem::zeroed();
    let mut d2: Vec3 = core::mem::zeroed();
    let mut attach_pos_1: Vec3 = core::mem::zeroed();
    let mut attach_pos_2: Vec3 = core::mem::zeroed();
    let mut attach_pos_3: Vec3 = core::mem::zeroed();
    let mut hitbox_data: itSamusGrapple_HitboxData = core::mem::zeroed();
    let mut m1: [[f32; 4]; 3] = core::mem::zeroed();
    let mut m2: [[f32; 4]; 3] = core::mem::zeroed();
    let mut m3: [[f32; 4]; 3] = core::mem::zeroed();
    let mut result: i32;
    let mut d: f32;

    if (*fp).motion_id == 0xD4 {
        if (*fp).mv.ss.unk7.x0 == ((*da).xA0 as f32) {
            it_802B9328_attach(link, &mut attach_pos_1, &mut m1);
        }
    } else if (*fp).motion_id == 0xD6 {
        if (*fp).mv.ss.unk7.x0 == ((*da).xB0 as f32) {
            it_802B9328_attach(link, &mut attach_pos_2, &mut m2);
        }
    } else if (*fp).motion_id == 0x165 {
        if (*fp).mv.ss.unk7.x0 == ((*da).xC0 as f32) {
            it_802B9328_attach(link, &mut attach_pos_3, &mut m3);
        }
    }

    if (*fp).ground_or_air == GA_Ground && (*grapple_ip).xDD4_itemVar.samusgrapple.x16 == 1 {
        if (*fp).input.pressed_buttons & 0x100 != 0 {
            hitbox_data.create_hitbox = it_803B8660();
            ext::ftColl_8007AFF8((*fp).gobj);
            it_802B7160((*fp).gobj, &mut hitbox_data);
            (*grapple_ip).xDD4_itemVar.samusgrapple.x16 += 1;
        } else {
            ext::ftColl_8007AFF8((*fp).gobj);
        }
    }

    next = (*link).next;

    if ((*fp).input.held_buttons[0] & 0x40) != 0 && (*grapple_ip).xDD4_itemVar.samusgrapple.x16 == 1 {
        let target = ext::ftCo_800A4A40(fp);
        if !target.is_null() {
            d2.x = (*target).cur_pos.x - (*link).pos.x;
            d2.y = (*target).cur_pos.y - (*link).pos.y;
            d2.z = (*target).cur_pos.z - (*link).pos.z;
            ext::lbVector_Normalize(&mut d2);
            (*link).vel.x = d2.x * (*attrs).x40;
            (*link).vel.y = d2.y * (*attrs).x40;
            (*link).vel.z = d2.z * (*attrs).x40;
        }
    }

    ext::it_802A4420(link);
    result = ext::it_802A3D90(link);
    if result != 0 {
        if result & 0x3F != 0 {
            result = 1;
            (*link).x1CC = (*link).coll_data.left_facing_wall.index;
        } else if result & 0xFC0 != 0 {
            result = 1;
            (*link).x1CC = (*link).coll_data.right_facing_wall.index;
        } else {
            result = 2;
        }
    }

    cur = link;
    while !next.is_null() {
        if (*next).x2C_b0() != 0 {
            (*next).vel.y -= it_802B9328_grav((*next).vel.y);
            (*next).pos.y += (*next).vel.y;
            d = ext::it_802A3C98(&mut (*next).pos, &mut (*cur).pos, &mut dir);
            if d > (*attrs).x38 {
                (*next).pos.x = fmadds(dir.x, (*attrs).x38, (*cur).pos.x);
                (*next).pos.y = fmadds(dir.y, (*attrs).x38, (*cur).pos.y);
                (*next).pos.z = fmadds(dir.z, (*attrs).x38, (*cur).pos.z);
            } else if d < (*attrs).x3C {
                (*next).pos.x = fmadds(dir.x, (*attrs).x3C, (*cur).pos.x);
                (*next).pos.y = fmadds(dir.y, (*attrs).x3C, (*cur).pos.y);
                (*next).pos.z = fmadds(dir.z, (*attrs).x3C, (*cur).pos.z);
            }
            ext::it_802A43EC(next);
        } else {
            if ext::it_802A3C98(pos, &mut (*cur).pos, &mut dir) > (*attrs).x38 {
                (*next).pos.x = fmadds(dir.x, (*attrs).x38, (*cur).pos.x);
                (*next).pos.y = fmadds(dir.y, (*attrs).x38, (*cur).pos.y);
                (*next).pos.z = fmadds(dir.z, (*attrs).x38, (*cur).pos.z);
                (*next).set_x2C_b0(1);
                ext::it_802A43B8(next);
            } else {
                return result;
            }
        }
        cur = next;
        next = (*next).next;
    }
    it_802B900C(cur, pos, attrs, (*attrs).x38);
    3
}

pub unsafe extern "C" fn it_802B99A0(link: *mut ItemLink, pos: *mut Vec3, attrs: *mut itSamusGrappleAttributes, fp: *mut Fighter) -> i32 {
    melee_core::ct!("it_802B99A0");
    let mut dir: Vec3 = core::mem::zeroed();
    let grapple_ip = (*(*fp).u.ss.x223C).user_data as *mut Item;
    let mut cur: *mut ItemLink;
    let mut next: *mut ItemLink;
    let mut result: i32;
    let mut d: f32;
    let mut hitbox_data: itSamusGrapple_HitboxData = core::mem::zeroed();

    if (*fp).ground_or_air == GA_Ground && (*grapple_ip).xDD4_itemVar.samusgrapple.x16 == 1 {
        if (*fp).input.pressed_buttons & 0x100 != 0 {
            hitbox_data.create_hitbox = it_803B8660();
            ext::ftColl_8007AFF8((*fp).gobj);
            it_802B7160((*fp).gobj, &mut hitbox_data);
            (*grapple_ip).xDD4_itemVar.samusgrapple.x16 += 1;
        } else {
            ext::ftColl_8007AFF8((*fp).gobj);
        }
    }

    next = (*link).next;
    cur = link;
    (*link).vel.y -= samus_grapple_calc_grav((*link).vel.y);
    if (*link).vel.x > (*attrs).x58 {
        (*link).vel.x -= (*attrs).x58;
    } else if (*link).vel.x < -(*attrs).x58 {
        (*link).vel.x += (*attrs).x58;
    } else {
        (*link).vel.x = 0.0;
    }

    ext::it_802A4420(cur);
    result = ext::it_802A3D90(cur);
    if result != 0 {
        if result & 0xFFF != 0 {
            result = 1;
        } else {
            result = 2;
        }
    }

    while !next.is_null() {
        if (*next).x2C_b0() != 0 {
            (*next).vel.y -= samus_grapple_calc_grav((*next).vel.y);
            ext::it_802A4420(next);
            d = ext::it_802A3C98(&mut (*next).pos, &mut (*cur).pos, &mut dir);
            if d > (*attrs).x38 {
                (*next).pos.x = fmadds(dir.x, (*attrs).x38, (*cur).pos.x);
                (*next).pos.y = fmadds(dir.y, (*attrs).x38, (*cur).pos.y);
                (*next).pos.z = fmadds(dir.z, (*attrs).x38, (*cur).pos.z);
            } else if d < (*attrs).x3C {
                (*next).pos.x = fmadds(dir.x, (*attrs).x3C, (*cur).pos.x);
                (*next).pos.y = fmadds(dir.y, (*attrs).x3C, (*cur).pos.y);
                (*next).pos.z = fmadds(dir.z, (*attrs).x3C, (*cur).pos.z);
            }
            ext::it_802A43EC(next);
        } else {
            if ext::it_802A3C98(pos, &mut (*cur).pos, &mut dir) > (*attrs).x38 {
                (*next).pos.x = fmadds(dir.x, (*attrs).x38, (*cur).pos.x);
                (*next).pos.y = fmadds(dir.y, (*attrs).x38, (*cur).pos.y);
                (*next).pos.z = fmadds(dir.z, (*attrs).x38, (*cur).pos.z);
                (*next).set_x2C_b0(1);
                ext::it_802A43B8(next);
            } else {
                return result;
            }
        }
        cur = next;
        next = (*next).next;
    }
    it_802B900C(cur, pos, attrs, (*attrs).x38);
    3
}

pub unsafe extern "C" fn it_802B9CE8(mut link: *mut ItemLink, pos: *mut Vec3, attrs: *mut itSamusGrappleAttributes, fp: *mut Fighter) {
    melee_core::ct!("it_802B9CE8");
    let mut prev: *mut ItemLink;
    let mut d: f32;
    let grapple_ip = (*(*fp).u.ss.x223C).user_data as *mut Item;
    let fp2: *mut Fighter;
    let mut dir: Vec3 = core::mem::zeroed();
    let mut hitbox_data: itSamusGrapple_HitboxData = core::mem::zeroed();

    fp2 = fp;
    if (*fp2).ground_or_air == GA_Ground && (*grapple_ip).xDD4_itemVar.samusgrapple.x16 == 1 {
        if (*fp2).input.pressed_buttons & 0x100 != 0 {
            hitbox_data.create_hitbox = it_803B8660();
            ext::ftColl_8007AFF8((*fp2).gobj);
            it_802B7160((*fp2).gobj, &mut hitbox_data);
            (*grapple_ip).xDD4_itemVar.samusgrapple.x16 += 1;
        } else {
            ext::ftColl_8007AFF8((*fp2).gobj);
        }
    }
    prev = (*link).prev;
    while !prev.is_null() && (*link).x2C_b0() == 0 {
        link = prev;
        prev = (*prev).prev;
    }
    (*link).vel.y -= samus_grapple_calc_grav((*link).vel.y);
    ext::it_802A4420(link);
    d = ext::it_802A3C98(&mut (*link).pos, pos, &mut dir);
    if d > (*attrs).x38 {
        (*link).pos.x = fmadds(dir.x, (*attrs).x38, (*pos).x);
        (*link).pos.y = fmadds(dir.y, (*attrs).x38, (*pos).y);
        (*link).pos.z = fmadds(dir.z, (*attrs).x38, (*pos).z);
    }
    while !prev.is_null() {
        (*prev).vel.y -= samus_grapple_calc_grav((*prev).vel.y);
        ext::it_802A4420(prev);
        d = ext::it_802A3C98(&mut (*prev).pos, &mut (*link).pos, &mut dir);
        if d > (*attrs).x38 {
            (*prev).pos.x = fmadds(dir.x, (*attrs).x38, (*link).pos.x);
            (*prev).pos.y = fmadds(dir.y, (*attrs).x38, (*link).pos.y);
            (*prev).pos.z = fmadds(dir.z, (*attrs).x38, (*link).pos.z);
        } else if d < (*attrs).x3C {
            (*prev).pos.x = fmadds(dir.x, (*attrs).x3C, (*link).pos.x);
            (*prev).pos.y = fmadds(dir.y, (*attrs).x3C, (*link).pos.y);
            (*prev).pos.z = fmadds(dir.z, (*attrs).x3C, (*link).pos.z);
        }
        link = prev;
        prev = (*prev).prev;
    }
    if (*link).vel.x > (*attrs).x58 {
        (*link).vel.x -= (*attrs).x58;
    } else if (*link).vel.x < -(*attrs).x58 {
        (*link).vel.x += (*attrs).x58;
    } else {
        (*link).vel.x = 0.0;
    }
}

pub unsafe extern "C" fn it_802B9FD4(link: *mut ItemLink, pos: *mut Vec3, attrs: *mut itSamusGrappleAttributes) -> bool {
    melee_core::ct!("it_802B9FD4");
    let mut cur: *mut ItemLink;
    let mut dir: Vec3 = core::mem::zeroed();
    let mut next: *mut ItemLink;

    ext::it_802A4454(link);
    cur = link;
    next = (*cur).next;

    while !next.is_null() {
        if (*next).x2C_b0() != 0 {
            (*next).vel.y -= samus_grapple_calc_grav((*next).vel.y);
            ext::it_802A4420(next);
            if ext::it_802A3C98(&mut (*next).pos, &mut (*cur).pos, &mut dir) > (*attrs).x38 {
                (*next).pos.x = fmadds(dir.x, (*attrs).x38, (*cur).pos.x);
                (*next).pos.y = fmadds(dir.y, (*attrs).x38, (*cur).pos.y);
                (*next).pos.z = fmadds(dir.z, (*attrs).x38, (*cur).pos.z);
            }
            ext::it_802A43EC(next);
        } else {
            if ext::it_802A3C98(pos, &mut (*cur).pos, &mut dir) > (*attrs).x38 {
                (*next).pos.x = fmadds(dir.x, (*attrs).x38, (*cur).pos.x);
                (*next).pos.y = fmadds(dir.y, (*attrs).x38, (*cur).pos.y);
                (*next).pos.z = fmadds(dir.z, (*attrs).x38, (*cur).pos.z);
                (*next).set_x2C_b0(1);
                ext::it_802A43B8(next);
            } else {
                return false;
            }
        }
        cur = next;
        next = (*cur).next;
    }
    true
}

pub unsafe extern "C" fn it_802BA194(link: *mut ItemLink, pos: *mut Vec3, attrs: *mut itSamusGrappleAttributes, target_dist: f32) -> bool {
    melee_core::ct!("it_802BA194");
    let mut next: *mut ItemLink;
    let mut cur: *mut ItemLink;
    let mut iter: *mut ItemLink;
    let mut d: f32;
    let mut remaining: f32;
    let result: bool;
    let mut dir: Vec3 = core::mem::zeroed();

    cur = link;
    next = (*link).prev;

    while !next.is_null() && (*cur).x2C_b0() == 0 {
        cur = next;
        next = (*next).prev;
    }

    d = ext::it_802A3C98(&mut (*cur).pos, pos, &mut dir);

    while !next.is_null() && target_dist > d {
        (*cur).set_x2C_b0(0);
        d = ext::it_802A3C98(&mut (*next).pos, pos, &mut dir);
        cur = next;
        next = (*next).prev;
    }

    d -= target_dist;
    if d > (*attrs).x38 {
        d = (*attrs).x38;
    }
    it_802B900C(cur, pos, attrs, d);

    if !next.is_null() {
        result = false;
    } else {
        result = true;
    }

    iter = link;
    while !iter.is_null() {
        let dx = (*iter).coll_data.cur_pos.x - (*iter).coll_data.last_pos.x;
        let dz = (*iter).coll_data.cur_pos.z - (*iter).coll_data.last_pos.z;
        remaining = dz - (*iter).vel.z;
        // `iter->vel.x += 0.12 * (dx - iter->vel.x)`: double product + add, contracted to an fma
        (*iter).vel.x = (0.12f64.fmadd((dx - (*iter).vel.x) as f64, (*iter).vel.x as f64)) as f32;
        (*iter).vel.z = (0.12f64.fmadd(remaining as f64, (*iter).vel.z as f64)) as f32;
        iter = (*iter).prev;
    }
    result
}

/// `Item_RetractChain` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_RetractChain(
    link: *mut ItemLink,
    pos: *mut Vec3,
    target_dist: f32,
    max_dist: *const f32,
    next_out: *mut *mut ItemLink,
    cur_out: *mut *mut ItemLink,
    remaining_out: *mut f32,
) {
    let mut cur: *mut ItemLink;
    let mut next: *mut ItemLink;
    let mut distance: f32;
    let mut remaining: f32;
    let limit: f32;
    let mut dir: Vec3 = core::mem::zeroed();

    cur = link;
    next = (*link).prev;

    while !next.is_null() && (*cur).x2C_b0() == 0 {
        cur = next;
        next = (*next).prev;
    }

    distance = ext::it_802A3C98(&mut (*cur).pos, pos, &mut dir);

    while !next.is_null() && target_dist > distance {
        (*cur).set_x2C_b0(0);
        distance = ext::it_802A3C98(&mut (*next).pos, pos, &mut dir);
        cur = next;
        next = (*next).prev;
    }

    remaining = distance - target_dist;
    limit = *max_dist;
    if remaining > limit {
        remaining = limit;
    }
    *next_out = next;
    *cur_out = cur;
    *remaining_out = remaining;
}

pub unsafe extern "C" fn it_802BA2D8(link: *mut ItemLink, pos: *mut Vec3, attrs: *mut itSamusGrappleAttributes, target_dist: f32) -> bool {
    melee_core::ct!("it_802BA2D8");
    let mut next: *mut ItemLink = null_mut();
    let mut cur: *mut ItemLink = null_mut();
    let mut remaining: f32 = 0.0;

    Item_RetractChain(link, pos, target_dist, &(*attrs).x38, &mut next, &mut cur, &mut remaining);
    it_802B900C(cur, pos, attrs, remaining);

    if !next.is_null() {
        return false;
    }
    true
}

pub unsafe extern "C" fn it_802BA3BC(
    mut tail: *mut ItemLink,
    mut head: *mut ItemLink,
    pos: *mut Vec3,
    attrs: *mut itSamusGrappleAttributes,
    target_dist: f32,
) -> bool {
    melee_core::ct!("it_802BA3BC");
    let mut dir: Vec3 = core::mem::zeroed();
    let saved_pos: Vec3;
    let mut link: *mut ItemLink;
    let mut next_tail: *mut ItemLink;
    let mut count: i32;
    let mut d: f32;
    let mut remaining: f32;
    let max_dist: f32;

    ext::it_802A4454(head);
    count = 0;
    next_tail = (*tail).prev;
    saved_pos = (*head).pos;

    while !next_tail.is_null() && (*tail).x2C_b0() == 0 {
        tail = next_tail;
        next_tail = (*next_tail).prev;
    }

    it_802B91C4(tail, pos, attrs, (*attrs).x38);
    (*head).pos = saved_pos;

    d = ext::it_802A3C98(&mut (*tail).pos, pos, &mut dir);

    while !next_tail.is_null() && target_dist > d {
        (*tail).set_x2C_b0(0);
        d = ext::it_802A3C98(&mut (*next_tail).pos, pos, &mut dir);
        tail = next_tail;
        next_tail = (*next_tail).prev;
    }

    remaining = d - target_dist;
    max_dist = (*attrs).x38;
    if remaining > max_dist {
        remaining = max_dist;
    }
    it_802B91C4(tail, pos, attrs, remaining);
    (*head).pos = saved_pos;

    link = (*head).next;
    while !link.is_null() && (*link).x2C_b0() != 0 {
        count += 1;
        if ext::it_802A3C98(&mut (*link).pos, &mut (*head).pos, &mut dir) > (*attrs).x38 {
            (*link).pos.x = fmadds(dir.x, (*attrs).x38, (*head).pos.x);
            (*link).pos.y = fmadds(dir.y, (*attrs).x38, (*head).pos.y);
            (*link).pos.z = fmadds(dir.z, (*attrs).x38, (*head).pos.z);
        }
        head = link;
        link = (*head).next;
    }

    if ext::it_802A3C98(pos, &mut (*head).pos, &mut dir) > (*attrs).x38 {
        (*pos).x = fmadds(dir.x, (*attrs).x38, (*head).pos.x);
        (*pos).y = fmadds(dir.y, (*attrs).x38, (*head).pos.y);
        (*pos).z = fmadds(dir.z, (*attrs).x38, (*head).pos.z);
    }

    if count != 0 {
        return false;
    }
    true
}

pub unsafe extern "C" fn it_802BA5DC(tail: *mut ItemLink, head: *mut ItemLink, pos: *mut Vec3, attrs: *mut itSamusGrappleAttributes) {
    melee_core::ct!("it_802BA5DC");
    let saved_pos: Vec3;
    let mut link: *mut ItemLink;
    let mut head_link: *mut ItemLink;
    let mut next: *mut ItemLink;
    let mut cur: *mut ItemLink;
    let mut retracted: bool;

    head_link = head;
    ext::it_802A4454(head_link);
    cur = tail;
    saved_pos = (*head_link).pos;
    next = (*tail).prev;

    while !next.is_null() && (*cur).x2C_b0() == 0 {
        cur = next;
        next = (*next).prev;
    }

    it_802B91C4(cur, pos, attrs, (*attrs).x38);
    retracted = false;
    (*head_link).pos = saved_pos;

    {
        let mut dir: Vec3 = core::mem::zeroed();

        link = (*head_link).next;
        while !link.is_null() && (*link).x2C_b0() != 0 {
            if !retracted {
                if ext::it_802A3C98(&mut (*link).pos, &mut (*head_link).pos, &mut dir) > (*attrs).x38 {
                    (*link).pos.x = fmadds(dir.x, (*attrs).x38, (*head_link).pos.x);
                    (*link).pos.y = fmadds(dir.y, (*attrs).x38, (*head_link).pos.y);
                    (*link).pos.z = fmadds(dir.z, (*attrs).x38, (*head_link).pos.z);
                } else {
                    retracted = true;
                }
            }
            head_link = link;
            link = (*link).next;
        }

        if ext::it_802A3C98(pos, &mut (*head_link).pos, &mut dir) > (*attrs).x38 {
            (*pos).x = fmadds(dir.x, (*attrs).x38, (*head_link).pos.x);
            (*pos).y = fmadds(dir.y, (*attrs).x38, (*head_link).pos.y);
            (*pos).z = fmadds(dir.z, (*attrs).x38, (*head_link).pos.z);
        }
    }
}

pub unsafe extern "C" fn it_802BA760(link: *mut ItemLink, pos: *mut Vec3, attrs: *mut itSamusGrappleAttributes, fp: *mut Fighter) -> bool {
    melee_core::ct!("it_802BA760");
    let mut dir: Vec3 = core::mem::zeroed();
    let da = (*(*fp).ft_data).ext_attr as *mut ftSs_DatAttrs;
    let mut cur: *mut ItemLink;
    let mut next: *mut ItemLink;

    if (*fp).motion_id == 0xD4 {
        if (*fp).mv.ss.unk7.x0 < ((*da).xA0 as f32) {
            return true;
        }
    } else if (*fp).motion_id == 0xD6 {
        if (*fp).mv.ss.unk7.x0 < ((*da).xB0 as f32) {
            return true;
        }
    } else if (*fp).motion_id == 0x165 {
        if (*fp).mv.ss.unk7.x0 < ((*da).xC0 as f32) {
            return true;
        }
    }
    cur = link;
    next = (*link).next;
    while !next.is_null() {
        if (*next).x2C_b0() != 0 {
            if ext::it_802A3C98(&mut (*next).pos, &mut (*cur).pos, &mut dir) > (*attrs).x38 {
                (*next).pos.x = fmadds(dir.x, (*attrs).x38, (*cur).pos.x);
                (*next).pos.y = fmadds(dir.y, (*attrs).x38, (*cur).pos.y);
                (*next).pos.z = fmadds(dir.z, (*attrs).x38, (*cur).pos.z);
            }
            ext::it_802A43EC(next);
        } else {
            if ext::it_802A3C98(pos, &mut (*cur).pos, &mut dir) > (*attrs).x38 {
                (*next).pos.x = fmadds(dir.x, (*attrs).x38, (*cur).pos.x);
                (*next).pos.y = fmadds(dir.y, (*attrs).x38, (*cur).pos.y);
                (*next).pos.z = fmadds(dir.z, (*attrs).x38, (*cur).pos.z);
                (*next).set_x2C_b0(1);
                ext::it_802A43B8(next);
            } else {
                return false;
            }
        }
        cur = next;
        next = (*next).next;
    }
    it_802B900C(cur, pos, attrs, (*attrs).x38);
    false
}

pub unsafe extern "C" fn itSamusGrapple_Logic53_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itSamusGrapple_Logic53_PickedUp");
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    ext::it_802A2428(gobj);
}

pub unsafe extern "C" fn it_802BA9B8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BA9B8");
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE);
    ext::ftColl_8007AFF8((*ip).xDD4_itemVar.samusgrapple.x8);
    ext::it_802A2428(gobj);
}

pub unsafe extern "C" fn it_802BAA08(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BAA08");
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
    ext::ftColl_8007AFF8((*ip).xDD4_itemVar.samusgrapple.x8);
    ext::it_802A2428(gobj);
}

pub unsafe extern "C" fn it_802BAA58(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BAA58");
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
    ext::it_802A2428(gobj);
}

pub unsafe extern "C" fn it_802BAA94(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BAA94");
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
    ext::ftColl_8007AFF8((*ip).xDD4_itemVar.samusgrapple.x8);
    ext::it_802A2428(gobj);
}

pub unsafe extern "C" fn it_802BAAE4(gobj: *mut Item_GObj, vel: *mut Vec3) {
    melee_core::ct!("it_802BAAE4");
    let ip = GET_ITEM(gobj);
    let link = (*ip).xDD4_itemVar.samusgrapple.x0;
    (*link).vel = *vel;
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    ext::it_802A2428(gobj);
}

pub unsafe extern "C" fn it_802BAB40(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BAB40");
    ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE);
    ext::it_802A2428(gobj);
}

pub unsafe extern "C" fn it_802BAB7C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BAB7C");
    ext::Item_80268E5C(gobj, 7, ITEM_ANIM_UPDATE);
    ext::it_802A2428(gobj);
}

/// Set grapple beam state and store duration.
pub unsafe extern "C" fn it_802BABB8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BABB8");
    let ip = (*gobj).user_data as *mut Item;
    let fp = (*(*ip).owner).user_data as *mut Fighter;
    let da = (*(*fp).ft_data).ext_attr as *mut ftSs_DatAttrs;
    ext::Item_80268E5C(gobj, 8, ITEM_ANIM_UPDATE);
    ext::it_802A2428(gobj);
    (*fp).mv.ss.grapple.x4 = (*da).xD0 as f32;
}

pub unsafe extern "C" fn it_802BAC3C(gobj: *mut Fighter_GObj) {
    melee_core::ct!("it_802BAC3C");
    let fp = GET_FIGHTER(gobj);
    if !(*fp).u.ss.x223C.is_null() {
        it_802B7B84((*fp).u.ss.x223C);
        return;
    }
    (*fp).accessory2_cb = None;
    (*fp).death1_cb = None;
    (*fp).accessory3_cb = None;
}

pub unsafe extern "C" fn it_802BAC80(gobj: *mut Fighter_GObj) {
    melee_core::ct!("it_802BAC80");
    let fp = GET_FIGHTER(gobj);
    if !(*fp).u.ss.x223C.is_null() {
        let ip = GET_ITEM((*fp).u.ss.x223C);
        if let Some(f) = (*ip).xDD4_itemVar.samusgrapple.unk_10 {
            f((*fp).u.ss.x223C);
        }
    }
}

pub unsafe extern "C" fn it_802BACC4(gobj: *mut Fighter_GObj) {
    melee_core::ct!("it_802BACC4");
    let fp = GET_FIGHTER(gobj);
    let mut pos: Vec3 = core::mem::zeroed();
    let mut m: [[f32; 4]; 3] = core::mem::zeroed();
    let mut m2: [[f32; 4]; 3] = core::mem::zeroed();

    if !(*fp).u.ss.x223C.is_null() {
        let ip = GET_ITEM((*fp).u.ss.x223C);
        let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusGrappleAttributes;
        let link = (*ip).xDD4_itemVar.samusgrapple.x0;

        samus_grapple_setup_pos(link, &mut pos, &mut m);

        if it_802BA760(link, &mut pos, attrs, fp) {
            fn_802B7E34_inline((*fp).u.ss.x223C, ip, &mut m2);
        } else {
            ext::it_802A7168(ip, &mut pos, (*fp).x34_scale.y);
        }
    }
}

pub unsafe extern "C" fn itSamusGrapple_Logic53_EvtUnk(gobj: *mut Item_GObj, arg1: *mut Item_GObj) {
    melee_core::ct!("itSamusGrapple_Logic53_EvtUnk");
    let item = GET_ITEM(gobj);
    ext::it_8026B894(gobj, arg1);
    if (*item).xDD4_itemVar.samusgrapple.x8 == arg1 {
        (*item).xDD4_itemVar.samusgrapple.x8 = null_mut();
    }
}

// ---- folded from itkinds6.rs (consolidation steps 21/22) ----
/// `samus_grapple_fighter_compare_fp` (itsamusgrapple.c, static inline): one of the four motions.
pub fn samus_grapple_fighter_compare_fp(id: i32) -> bool {
    melee_core::ct!("samus_grapple_fighter_compare_fp");
    id == 0x165 || id == 0x166 || id == 0xD4 || id == 0xD6
}

// ---- folded from itkinds9.rs (consolidation steps 21/22) ----
/// `samus_grapple_calc_grav_fp` (itsamusgrapple.c). Both comparisons are in f64 (`0.9`, `0.0`).
pub fn samus_grapple_calc_grav_fp(rand: &mut HsdRand, vel_y: f32) -> f32 {
    melee_core::ct!("samus_grapple_calc_grav_fp");
    if rand.randf() as f64 > 0.9 {
        if (vel_y as f64) < 0.0 {
            -fmadds(0.6, rand.randf(), -vel_y)
        } else {
            fmadds(0.6, rand.randf(), vel_y)
        }
    } else {
        0.0
    }
}

/// `it_802B9328_grav` (itsamusgrapple.c): as `samus_grapple_calc_grav_fp` with factor 1.
pub fn it_802b9328_grav(rand: &mut HsdRand, vely: f32) -> f32 {
    melee_core::ct!("it_802b9328_grav");
    let one = 1.0f32;
    if rand.randf() as f64 > 0.9 {
        if (vely as f64) < 0.0 {
            -fmadds(one, rand.randf(), -vely)
        } else {
            fmadds(one, rand.randf(), vely)
        }
    } else {
        0.0
    }
}
