//! Port of src/melee/it/kinds/itseakneedlethrown.c: Sheik's thrown needle (logic table 109). Callees of
//! unported TUs go through `ext` (unset: panics "TODO"); `sinf` / `cosf` are the `melee_runtime::gekko`
//! ports. Checked against the C (whole TU, shipping contraction) by tests/kinds_itseakneedlethrown.rs.
//!
//! Fused sites (`fmadds`, f64 multiply + f64 add + f32 round): `facing_dir * angle_factor + pi/2` and
//! `pos - 3.0 * vel` in `it_802AFF08`. `ABS(table[HSD_Randi(8)])` evaluates its argument twice (the macro
//! names it in the test and in the result), so it draws two random numbers.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_mut, unused_variables, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::gekko::{cosf, sinf};
use melee_types::*;

use super::kinds_inl::*;

// // BEGIN GENERATED EXT (rust/melee_items/gen/kinds/gen_tu.py; the callees of the C TU)
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
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn ftLib_800865CC(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ftlib::ftLib_800865CC(a0) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026BD6C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd6c(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026EA20(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec, a4: *mut i32, a5: *mut u32) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026EA20(a0, a1 as _, a2 as _, a3 as _, a4, a5) };
    fn it_80272560(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::it_2725::it_80272560(a0, a1) };
    fn it_80272940(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272940(a0) };
    fn it_80273130(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273130(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn lbVector_Mirror(a0: *mut Vec, a1: *mut Vec) => unsafe { melee_runtime::tu::lbvector::lbVector_Mirror(a0, a1) };
    fn mpColl_80043558(a0: *mut CollData, a1: i32) => unsafe { melee_world::tu::mpcoll::mpColl_80043558(a0, a1) };
    fn mpGetSpeed(a0: i32, a1: *mut Vec, a2: *mut Vec) -> i32 => unsafe { melee_world::tu::mplib::mpGetSpeed(a0, a1, a2) };
    fn mpLib_80054ED8(a0: i32) -> i32 => unsafe { melee_world::tu::mplib::mpLib_80054ED8(a0) };
    fn mpLineGetNormal(a0: i32, a1: *mut Vec) -> *mut Vec => unsafe { melee_world::tu::mplib::mpLineGetNormal(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn HSD_Randi(a0: i32) -> i32 { super::HSD_Randi(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

const M_PI_2: f64 = 3.14159265358979323846 / 2.0;
const M_TAU: f64 = 6.283185307179586;

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
    ext::it_8026BB68(parent, &mut (*spawn).pos);
}

/// `Item_ClearCmdVars` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_ClearCmdVars(ip: *mut Item) {
    (*ip).xDB8_itcmd_var3 = 0;
    (*ip).xDB4_itcmd_var2 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDAC_itcmd_var0 = 0;
}

/// `ItemStateTable it_803F6F50[]`
pub static mut it_803F6F50: [ItemStateTable; 5] = [
    st_row(0, pred_b(itSeakneedlethrown_UnkMotion0_Anim), Some(itSeakneedlethrown_UnkMotion0_Phys), pred_b(itSeakneedlethrown_UnkMotion0_Coll)),
    st_row(1, pred_b(itSeakneedlethrown_UnkMotion1_Anim), Some(itSeakneedlethrown_UnkMotion1_Phys), pred_b(itSeakneedlethrown_UnkMotion1_Coll)),
    st_row(2, pred_b(itSeakneedlethrown_UnkMotion2_Anim), Some(itSeakneedlethrown_UnkMotion2_Phys), pred_b(itSeakneedlethrown_UnkMotion2_Coll)),
    st_row(3, pred_b(itSeakneedlethrown_UnkMotion3_Anim), Some(itSeakneedlethrown_UnkMotion3_Phys), pred_b(itSeakneedlethrown_UnkMotion3_Coll)),
    st_row(4, pred_b(itSeakneedlethrown_UnkMotion4_Anim), Some(itSeakneedlethrown_UnkMotion4_Phys), pred_b(itSeakneedlethrown_UnkMotion4_Coll)),
];
static it_803F6FA0: [f32; 8] = [-2.0, -2.1, -2.2, -2.3, -2.4, -2.5, -2.6, -2.7];
static it_803F6FC0: [f32; 8] = [-0.1, -0.12, -0.14, -0.18, -0.2, -0.22, -0.24, -0.26];
static it_803F6FE0: [f32; 8] = [0.20943952, 0.2443461, 0.27925268, 0.31415927, 0.34906584, 0.38397244, 0.41887903, 0.4537856];
static it_803F7000: [f32; 8] = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4];
static it_803F7020: [f32; 8] = [-2.0, -2.1, -2.2, -2.3, -2.4, -2.5, -2.6, -2.7];
static it_803F7040: [f32; 8] = [-0.1, -0.12, -0.14, -0.18, -0.2, -0.22, -0.24, -0.26];
static it_803F7060: [f32; 8] = [0.5235988, 0.61086524, 0.6981317, 0.7853982, 0.87266463, 0.9599311, 1.0471976, 1.134464];

/// `ABS(it_803F7020[HSD_Randi(8)])`: the macro names its argument twice (test and result).
#[inline]
unsafe fn abs_table_7020() -> f32 {
    let x = it_803F7020[ext::HSD_Randi(8) as usize];
    if x < 0.0 {
        -it_803F7020[ext::HSD_Randi(8) as usize]
    } else {
        it_803F7020[ext::HSD_Randi(8) as usize]
    }
}

pub unsafe extern "C" fn it_802AFD8C(parent: *mut Item_GObj, pos: *mut Vec3, kind: u32, facing_dir: f32) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = kind as _;
    Item_InitSpawnPositionFromParent(&mut spawn, parent, pos);
    Item_InitSpawnCommonFields(&mut spawn, parent, facing_dir, true);
    let item_gobj = ext::Item_80268B18(&mut spawn);
    if !item_gobj.is_null() {
        let ip = GET_ITEM(item_gobj);
        let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSeakNeedleThrownAttributes;
        Item_ClearCmdVars(ip);
        ext::it_80275158(item_gobj, (*attr).x0);
        (*ip).xDD4_itemVar.seakneedlethrown.xDF4 = -1;
        (*ip).xDD4_itemVar.seakneedlethrown.xDFC = 0.0;
        (*ip).xDD4_itemVar.seakneedlethrown.xDF8 = 0.0;
        (*ip).xDD4_itemVar.seakneedlethrown.xDE4 = (*ip).pos;
        ext::it_8026BD6C(item_gobj);
    }
    item_gobj
}

pub unsafe extern "C" fn it_802AFEA8(gobj: *mut Item_GObj, owner_gobj: *mut Fighter_GObj, arg2: u32) {
    if !gobj.is_null() && !owner_gobj.is_null() {
        match arg2 {
            0 => it_802AFF08(gobj, owner_gobj),
            1 => it_802B00F4(gobj, owner_gobj),
            _ => {}
        }
    }
}

pub unsafe extern "C" fn itSeakNeedleThrown_Logic109_Destroyed(gobj: *mut Item_GObj) {
    (*GET_ITEM(gobj)).owner = null_mut();
}

pub unsafe extern "C" fn it_802AFF08(gobj: *mut Item_GObj, owner: *mut Fighter_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSeakNeedleThrownAttributes;
    let child = HSD_JObjGetChild(GET_JOBJ(gobj));
    let angle_factor: f32;

    (*ip).owner = owner;
    ext::it_8026B3A8(gobj);
    (*ip).xDC8_word.flags.set_x13(0);
    ext::it_80272940(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    ext::it_8026BD6C(gobj);
    ext::it_80275158(gobj, (*attr).x0);
    if ext::ftLib_800865CC(owner) != 0 {
        angle_factor = 2.3561945;
    } else {
        angle_factor = 1.5707964;
    }
    (*ip).xDD4_itemVar.seakneedlethrown.xDF0 = fmadds((*ip).facing_dir, angle_factor, 1.5707964);
    (*ip).x40_vel.x = -(*attr).x8 * cosf((*ip).xDD4_itemVar.seakneedlethrown.xDF0);
    (*ip).x40_vel.y = (*attr).x8 * sinf((*ip).xDD4_itemVar.seakneedlethrown.xDF0);
    (*ip).x40_vel.z = 0.0;
    (*ip).xDD4_itemVar.seakneedlethrown.xDE4.x = fmadds(-3.0, (*ip).x40_vel.x, (*ip).pos.x);
    (*ip).xDD4_itemVar.seakneedlethrown.xDE4.y = fmadds(-3.0, (*ip).x40_vel.y, (*ip).pos.y);
    (*ip).xDD4_itemVar.seakneedlethrown.xDE4.z = 0.0;
    HSD_JObjSetRotationX(
        child,
        -(*ip).facing_dir
            * ext::atan2f(
                (*ip).pos.y - (*ip).xDD4_itemVar.seakneedlethrown.xDE4.y,
                (*ip).pos.x - (*ip).xDD4_itemVar.seakneedlethrown.xDE4.x,
            ),
    );
    ext::db_80225DD8(gobj, owner);
}

/// `itSeakNeedleThrown_SetupBounce` (static inline)
#[inline]
unsafe fn itSeakNeedleThrown_SetupBounce(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let mut rand_sign: f32 = if ext::HSD_Randi(2) == 0 { 1.0 } else { -1.0 };
    let mut v = it_803F7060[ext::HSD_Randi(8) as usize];
    (*ip).xDD4_itemVar.seakneedlethrown.xDD4 = v * rand_sign;
    rand_sign = if ext::HSD_Randi(2) == 0 { 1.0 } else { -1.0 };
    v = it_803F7000[ext::HSD_Randi(8) as usize];
    (*ip).xDD4_itemVar.seakneedlethrown.xDD8 = v * rand_sign;
    (*ip).xDD4_itemVar.seakneedlethrown.xDDC = it_803F7020[ext::HSD_Randi(8) as usize];
    (*ip).xDD4_itemVar.seakneedlethrown.xDE0 = it_803F7040[ext::HSD_Randi(8) as usize];
}

/// `itSeakNeedleThrown_SetupDrop` (static inline)
#[inline]
unsafe fn itSeakNeedleThrown_SetupDrop(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let a = it_803F6FE0[ext::HSD_Randi(8) as usize];
    let b: f32 = if ext::HSD_Randi(2) == 0 { 1.0 } else { -1.0 };
    (*ip).xDD4_itemVar.seakneedlethrown.xDD4 = a * b;
    (*ip).xDD4_itemVar.seakneedlethrown.xDD8 = 0.0;
    (*ip).xDD4_itemVar.seakneedlethrown.xDDC = it_803F6FA0[ext::HSD_Randi(8) as usize];
    (*ip).xDD4_itemVar.seakneedlethrown.xDE0 = it_803F6FC0[ext::HSD_Randi(8) as usize];
}

pub unsafe extern "C" fn it_802B00F4(gobj: *mut Item_GObj, owner: *mut Fighter_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSeakNeedleThrownAttributes;

    (*ip).owner = owner;
    ext::it_8026B3A8(gobj);
    (*ip).xDC8_word.flags.set_x13(0);
    ext::it_80272940(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    ext::it_8026BD6C(gobj);
    ext::it_80275158(gobj, (*attr).x0);
    itResetVelocity(ip);

    itSeakNeedleThrown_SetupDrop(gobj);
    ext::db_80225DD8(gobj, owner);
}

pub unsafe extern "C" fn itSeakneedlethrown_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.seakneedlethrown.xDE4 = (*ip).pos;
    ext::it_80273130(gobj) != 0
}

pub unsafe extern "C" fn itSeakneedlethrown_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = HSD_JObjGetChild(GET_JOBJ(gobj));
    (*ip).xDD4_itemVar.seakneedlethrown.xDE4 = (*ip).pos;
    HSD_JObjAddRotationX(jobj, (*ip).xDD4_itemVar.seakneedlethrown.xDD4);
    ext::it_80273130(gobj) != 0
}

pub unsafe extern "C" fn itSeakneedlethrown_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let child = HSD_JObjGetChild((*gobj).hsd_obj as *mut HSD_JObj);
    (*ip).xDD4_itemVar.seakneedlethrown.xDE4 = (*ip).pos;
    HSD_JObjSetRotationX(child, (*ip).xDD4_itemVar.seakneedlethrown.xDF0);
    ext::it_80273130(gobj) != 0
}

pub unsafe extern "C" fn itSeakneedlethrown_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    ext::it_80273130(gobj) != 0
}

pub unsafe extern "C" fn itSeakneedlethrown_UnkMotion4_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = HSD_JObjGetChild(GET_JOBJ(gobj));
    (*ip).xDD4_itemVar.seakneedlethrown.xDE4 = (*ip).pos;
    HSD_JObjAddRotationX(jobj, (*ip).xDD4_itemVar.seakneedlethrown.xDD4);
    false
}

pub unsafe extern "C" fn itSeakneedlethrown_UnkMotion0_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itSeakneedlethrown_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.x = (*ip).xDD4_itemVar.seakneedlethrown.xDD8;
    (*ip).x40_vel.y += (*ip).xDD4_itemVar.seakneedlethrown.xDE0;
    if (*ip).x40_vel.y < (*ip).xDD4_itemVar.seakneedlethrown.xDDC {
        (*ip).x40_vel.y = (*ip).xDD4_itemVar.seakneedlethrown.xDDC;
    }
}

pub unsafe extern "C" fn itSeakneedlethrown_UnkMotion2_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itSeakneedlethrown_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    itResetVelocity(ip);
}

pub unsafe extern "C" fn itSeakneedlethrown_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.x = (*ip).xDD4_itemVar.seakneedlethrown.xDD8;
    (*ip).x40_vel.y += (*ip).xDD4_itemVar.seakneedlethrown.xDE0;
    if (*ip).x40_vel.y < (*ip).xDD4_itemVar.seakneedlethrown.xDDC {
        (*ip).x40_vel.y = (*ip).xDD4_itemVar.seakneedlethrown.xDDC;
    }
}

/// `itSeakNeedleThrown_CheckGroundHit` / `..._CheckGroundHit4` (identical static inlines)
#[inline]
unsafe fn itSeakNeedleThrown_CheckGroundHit(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let coll: *mut CollData = &mut (*ip).x378_itemColl;
    let mut line_id: u32 = 0;
    let mut hit_pos: Vec3 = core::mem::zeroed();

    if ext::it_8026EA20(
        gobj,
        &mut (*ip).xDD4_itemVar.seakneedlethrown.xDE4,
        &mut (*ip).pos,
        &mut hit_pos,
        &mut (*ip).xDD4_itemVar.seakneedlethrown.xDF4,
        &mut line_id,
    ) != 0
        && ext::mpLib_80054ED8((*ip).xDD4_itemVar.seakneedlethrown.xDF4) != 0
    {
        ext::mpColl_80043558(coll, (*ip).xDD4_itemVar.seakneedlethrown.xDF4);
        ext::mpGetSpeed((*ip).xDD4_itemVar.seakneedlethrown.xDF4, &mut (*ip).pos, &mut (*ip).x40_vel);
        {
            let angle = ext::atan2f(hit_pos.y, hit_pos.x);
            (*ip).xDD4_itemVar.seakneedlethrown.xDFC = angle;
            (*ip).xDD4_itemVar.seakneedlethrown.xDF8 = angle;
        }
        return true;
    }
    false
}

pub unsafe extern "C" fn itSeakneedlethrown_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = (*gobj).user_data as *mut Item;
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSeakNeedleThrownAttributes;
    let child = HSD_JObjGetChild((*gobj).hsd_obj as *mut HSD_JObj);
    let rot = -(*ip).facing_dir
        * ext::atan2f(
            (*ip).pos.y - (*ip).xDD4_itemVar.seakneedlethrown.xDE4.y,
            (*ip).pos.x - (*ip).xDD4_itemVar.seakneedlethrown.xDE4.x,
        );
    HSD_JObjSetRotationX(child, rot);
    if itSeakNeedleThrown_CheckGroundHit(gobj) {
        match ext::HSD_Randi(5) {
            0 | 1 | 2 => {
                ext::it_802762BC(ip);
                itResetVelocity(ip);
                ext::it_80275158(gobj, (*attr).x4);
                (*ip).xDD4_itemVar.seakneedlethrown.xDF0 = rot;
                ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
                ext::Item_8026AE84(ip, 0x41F39, 127, 64);
            }
            3 | 4 => {
                ext::it_80275158(gobj, (*attr).x4);
                (*ip).x40_vel.y = abs_table_7020();
                ext::it_802762BC(ip);
                itSeakNeedleThrown_SetupBounce(gobj);
                ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
            }
            _ => {}
        }
    }
    false
}

pub unsafe extern "C" fn itSeakneedlethrown_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = (*gobj).user_data as *mut Item;
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSeakNeedleThrownAttributes;
    if itSeakNeedleThrown_CheckGroundHit(gobj) {
        ext::it_80275158(gobj, (*attr).x4);
        let y = (*ip).x40_vel.y;
        (*ip).x40_vel.y = if y < 0.0 { -y } else { y };
        ext::it_802762BC(ip);
        itSeakNeedleThrown_SetupBounce(gobj);
        ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
        ext::Item_8026AE84(ip, 0x41F39, 127, 64);
    }
    false
}

/// `itSeakNeedleThrown_Coll2_Inline` (static inline)
#[inline]
unsafe fn itSeakNeedleThrown_Coll2_Inline(gobj: *mut Item_GObj, ip: *mut Item) {
    let coll: *mut CollData = &mut (*ip).x378_itemColl;
    let mut normal: Vec3 = core::mem::zeroed();

    (*ip).xDD4_itemVar.seakneedlethrown.xDFC = (*ip).xDD4_itemVar.seakneedlethrown.xDF8;
    if (*ip).xDD4_itemVar.seakneedlethrown.xDF4 != -1 {
        if ext::mpLib_80054ED8((*ip).xDD4_itemVar.seakneedlethrown.xDF4) != 0 {
            ext::mpColl_80043558(coll, (*ip).xDD4_itemVar.seakneedlethrown.xDF4);
            ext::mpGetSpeed((*ip).xDD4_itemVar.seakneedlethrown.xDF4, &mut (*ip).pos, &mut (*ip).x40_vel);
            ext::mpLineGetNormal((*ip).xDD4_itemVar.seakneedlethrown.xDF4, &mut normal);
            (*ip).xDD4_itemVar.seakneedlethrown.xDF8 = ext::atan2f(normal.y, normal.x);
        }
    } else {
        itResetVelocity(ip);
    }
}

/// `itSeakNeedleThrown_Coll2_Rotate` (static inline)
#[inline]
unsafe fn itSeakNeedleThrown_Coll2_Rotate(ip: *mut Item, child: *mut HSD_JObj) {
    if (*ip).xDD4_itemVar.seakneedlethrown.xDF8 != (*ip).xDD4_itemVar.seakneedlethrown.xDFC {
        (*ip).xDD4_itemVar.seakneedlethrown.xDF0 +=
            (*ip).xDD4_itemVar.seakneedlethrown.xDF8 - (*ip).xDD4_itemVar.seakneedlethrown.xDFC;
        HSD_JObjSetRotationX(child, (*ip).xDD4_itemVar.seakneedlethrown.xDF0);
    }
}

pub unsafe extern "C" fn itSeakneedlethrown_UnkMotion2_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let child = HSD_JObjGetChild((*gobj).hsd_obj as *mut HSD_JObj);
    ext::it_802762BC(ip);
    itResetVelocity(ip);
    itSeakNeedleThrown_Coll2_Inline(gobj, GET_ITEM(gobj));
    itSeakNeedleThrown_Coll2_Rotate(ip, child);
    false
}

pub unsafe extern "C" fn itSeakneedlethrown_UnkMotion3_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itSeakneedlethrown_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSeakNeedleThrownAttributes;
    let child = HSD_JObjGetChild((*gobj).hsd_obj as *mut HSD_JObj);
    if itSeakNeedleThrown_CheckGroundHit(gobj) {
        ext::it_80275158(gobj, (*attr).x4);
        (*ip).xDD4_itemVar.seakneedlethrown.xDF0 = HSD_JObjGetRotationX(child);
        ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
        ext::it_802762BC(ip);
        itResetVelocity(ip);
    }
    false
}

/// The shared body of `DmgDealt` / `Clanked` / `DmgReceived` / `HitShield`.
#[inline]
unsafe fn logic109_bounce_or_true(gobj: *mut Item_GObj, ip: *mut Item) -> bool {
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSeakNeedleThrownAttributes;

    if ext::HSD_Randi(3) == 0 {
        ext::it_80272560(gobj, 0);
        ext::it_80275158(gobj, (*attr).x4);
        (*ip).x40_vel.z = 0.0;
        (*ip).x40_vel.x = 0.0;
        (*ip).x40_vel.y = abs_table_7020();
        ext::it_802762BC(ip);
        itSeakNeedleThrown_SetupBounce(gobj);
        ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
        return false;
    }
    true
}

pub unsafe extern "C" fn it_2725_Logic109_DmgDealt(gobj: *mut Item_GObj) -> bool {
    logic109_bounce_or_true(gobj, GET_ITEM(gobj))
}

pub unsafe extern "C" fn it_2725_Logic109_Clanked(gobj: *mut Item_GObj) -> bool {
    logic109_bounce_or_true(gobj, GET_ITEM(gobj))
}

pub unsafe extern "C" fn it_2725_Logic109_DmgReceived(gobj: *mut Item_GObj) -> bool {
    logic109_bounce_or_true(gobj, GET_ITEM(gobj))
}

pub unsafe extern "C" fn it_2725_Logic109_Reflected(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSeakNeedleThrownAttributes;

    (*ip).facing_dir = -(*ip).facing_dir;
    HSD_JObjSetRotationY(jobj, (M_PI_2 * (*ip).facing_dir as f64) as f32);

    let dx = (*ip).pos.x - (*ip).xDD4_itemVar.seakneedlethrown.xDE4.x;
    let dy = (*ip).pos.y - (*ip).xDD4_itemVar.seakneedlethrown.xDE4.y;
    (*ip).xDD4_itemVar.seakneedlethrown.xDE4.x = (*ip).pos.x + dx;
    (*ip).xDD4_itemVar.seakneedlethrown.xDE4.y = (*ip).pos.y + dy;
    (*ip).xDD4_itemVar.seakneedlethrown.xDE4.z = 0.0;

    {
        let rot = -(*ip).facing_dir
            * ext::atan2f(
                (*ip).pos.y - (*ip).xDD4_itemVar.seakneedlethrown.xDE4.y,
                (*ip).pos.x - (*ip).xDD4_itemVar.seakneedlethrown.xDE4.x,
            );
        HSD_JObjSetRotationX(HSD_JObjGetChild(jobj), rot);
    }

    let angle = ext::atan2f(
        (*ip).pos.y - (*ip).xDD4_itemVar.seakneedlethrown.xDE4.y,
        (*ip).pos.x - (*ip).xDD4_itemVar.seakneedlethrown.xDE4.x,
    );
    (*ip).x40_vel.x = (*attr).x8 * cosf(angle);
    (*ip).x40_vel.y = (*attr).x8 * sinf(angle);
    (*ip).xD44_lifeTimer = (*ip).xD48_halfLifeTimer;

    false
}

pub unsafe extern "C" fn it_2725_Logic109_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    ext::lbVector_Mirror(&mut (*ip).x40_vel, &mut (*ip).xC58);
    let mut angle: f32 = ext::atan2f((*ip).x40_vel.y, (*ip).x40_vel.x);
    while angle < 0.0 {
        angle = (angle as f64 + M_TAU) as f32;
    }
    while angle as f64 > M_TAU {
        angle = (angle as f64 - M_TAU) as f32;
    }
    (*ip).facing_dir = if (*ip).x40_vel.x >= 0.0 { 1.0 } else { -1.0 };

    HSD_JObjSetRotationX(HSD_JObjGetChild(jobj), angle * -(*ip).facing_dir);
    HSD_JObjSetRotationY(jobj, (M_PI_2 * (*ip).facing_dir as f64) as f32);
    false
}

pub unsafe extern "C" fn it_2725_Logic109_HitShield(gobj: *mut Item_GObj) -> bool {
    logic109_bounce_or_true(gobj, GET_ITEM(gobj))
}

pub unsafe extern "C" fn itSeakNeedleThrown_Logic109_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
