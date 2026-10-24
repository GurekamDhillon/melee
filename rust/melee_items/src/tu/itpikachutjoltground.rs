//! Port of src/melee/it/kinds/itpikachutjoltground.c: the ground half of Pikachu's Thunder Jolt (logic
//! table 106), which walks along the floor and hands over to the air half (itpikachutjoltair) at
//! corners. `xDD4` is the travel angle (f32 field, computed in f64), `xDDC` the linked air half.
//! `atan2f` is an `ext` hook, `sinf` / `cosf` the gekko ports. Checked against the C (whole TU,
//! shipping contraction) by tests/kinds_itpikachutjoltground.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_runtime::gekko::{cosf, sinf};
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
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn efLib_DestroyAll(a0: *mut HSD_GObj);
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj) -> *mut c_void;
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026E9A4(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026E9A4(a0, a1 as _, a2 as _, a3 as _) };
    fn it_802725D4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802725D4(a0) };
    fn it_80272940(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272940(a0) };
    fn it_80273130(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273130(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_802B3EFC(a0: *mut HSD_GObj) -> *mut HSD_GObj => unsafe { crate::tu::itpikachutjoltair::it_802B3EFC(a0 as _) as _ };
    fn it_802B3F20(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::itpikachutjoltair::it_802B3F20(a0 as _, a1 as _) };
    fn it_802B3F88(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut CollData, a3: *mut Vec) => unsafe { crate::tu::itpikachutjoltair::it_802B3F88(a0 as _, a1 as _, a2, a3 as _) };
    fn it_802B4224(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: *mut Vec, a3: i32, a4: f32) -> *mut HSD_GObj => unsafe { crate::tu::itpikachutjoltair::it_802B4224(a0, a1 as _, a2 as _, a3, a4) as _ };
    fn it_802B43B0(a0: *mut HSD_GObj) => unsafe { crate::tu::itpikachutjoltair::it_802B43B0(a0 as _) };
    fn lbVector_Angle(a0: *mut Vec, a1: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_Angle(a0, a1) };
    fn lbVector_Mirror(a0: *mut Vec, a1: *mut Vec) => unsafe { melee_runtime::tu::lbvector::lbVector_Mirror(a0, a1) };
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

const M_PI: f64 = core::f64::consts::PI;
const M_PI_2: f64 = core::f64::consts::FRAC_PI_2;
const M_TAU: f64 = 6.283185307179586;

/// `ItemStateTable it_803F7190[]`
pub static mut it_803F7190: [ItemStateTable; 2] = [
    st_row(
        0,
        pred_b(itPikachutjoltground_UnkMotion0_Anim),
        Some(itPikachutjoltground_UnkMotion0_Phys),
        pred_b(itPikachutjoltground_UnkMotion0_Coll),
    ),
    st_row(
        1,
        pred_b(itPikachutjoltground_UnkMotion1_Anim),
        Some(itPikachutjoltground_UnkMotion1_Phys),
        pred_b(itPikachutjoltground_UnkMotion1_Coll),
    ),
];

/// 2B3368
pub unsafe extern "C" fn it_802B3368(gobj: *mut Item_GObj) -> *mut Item_GObj {
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        if !ip.is_null() {
            return (*ip).xDD4_itemVar.pikachujoltground.xDDC;
        }
    }
    null_mut()
}

pub unsafe extern "C" fn itPikachuThunderJolt_Spawn(owner: *mut Item_GObj, pos: *mut Vec3, facing_dir: f32, kind: ItemKind) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    let mut coll_normal: Vec3 = core::mem::zeroed();

    spawn.kind = kind;
    // Item_InitSpawnPositionFromParent(&spawn, owner, pos)
    spawn.prev_pos = *pos;
    spawn.prev_pos.z = 0.0;
    ext::it_8026BB68(owner, &mut spawn.pos);
    // Item_InitSpawnCommonFields(&spawn, owner, facing_dir, true)
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = owner;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;
    let item_gobj = ext::Item_80268B18(&mut spawn);
    if !item_gobj.is_null() {
        let ip = GET_ITEM(item_gobj);
        let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPikachutJoltGroundAttributes;
        if ext::it_8026E9A4(item_gobj, &mut spawn.pos, &mut spawn.prev_pos, &mut coll_normal) != 0 {
            (*ip).pos = spawn.pos;
        }
        // Item_ClearCmdVars(ip)
        (*ip).xDB8_itcmd_var3 = 0;
        (*ip).xDB4_itcmd_var2 = 0;
        (*ip).xDB0_itcmd_var1 = 0;
        (*ip).xDAC_itcmd_var0 = 0;
        ext::it_80275158(item_gobj, (*attr).x0);
        (*ip).xDD4_itemVar.pikachujoltground.xDDC = null_mut();
        (*ip).xDD4_itemVar.pikachujoltground.xDE4 = 0;
        (*ip).xDD4_itemVar.pikachujoltground.xDE8 = spawn.pos;
        (*ip).xDD4_itemVar.pikachujoltground.xDF4.z = 0.0;
        (*ip).xDD4_itemVar.pikachujoltground.xDF4.y = 0.0;
        (*ip).xDD4_itemVar.pikachujoltground.xDF4.x = 0.0;
        it_802B3554(item_gobj, owner);
        (*ip).xDD4_itemVar.pikachujoltground.xDD8 = owner;
    }
    item_gobj
}

/// shared body of the Destroyed / Clanked / Absorbed / HitShield / DmgDealt callbacks
#[inline]
unsafe fn unlink_and_destroy(gobj: *mut Item_GObj) {
    let mut ip = GET_ITEM(gobj);
    if !(*ip).xDD4_itemVar.pikachujoltground.xDDC.is_null() {
        ext::it_802B43B0((*ip).xDD4_itemVar.pikachujoltground.xDDC);
        (*ip).xDD4_itemVar.pikachujoltground.xDDC = null_mut();
    }
    ip = GET_ITEM(gobj);
    ext::efLib_DestroyAll(gobj);
    (*ip).xDD4_itemVar.pikachujoltground.xDE0 = 0;
}

pub unsafe extern "C" fn it_2725_Logic106_Destroyed(gobj: *mut Item_GObj) {
    unlink_and_destroy(gobj);
}

/// 2B3544
pub unsafe extern "C" fn it_802B3544(gobj: *mut Item_GObj) {
    (*GET_ITEM(gobj)).xDD4_itemVar.pikachujoltground.xDDC = null_mut();
}

/// 2B3554
pub unsafe extern "C" fn it_802B3554(gobj: *mut Item_GObj, owner: *mut HSD_GObj) {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPikachutJoltGroundAttributes;

    // Item_ClearFlagsAndEnterState(gobj, ip, 0)
    ext::it_8026B3A8(gobj);
    (*ip).xDC8_word.flags.set_x13(0);
    ext::it_80272940(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);

    let angle: f64 = if (*ip).facing_dir == 1.0 {
        (*attr).x4 as f64
    } else {
        // ABS(attr->x4)
        let a = (*attr).x4;
        M_PI + (if a < 0.0 { -a } else { a }) as f64
    };
    (*ip).xDD4_itemVar.pikachujoltground.xDD4 = angle as f32;

    match (*ip).kind {
        k if k == It_Kind_Pikachu_TJolt_Ground || k == It_Kind_Pichu_TJolt_Ground => {
            ext::efSync_Spawn(0x4BD, gobj, jobj);
            (*ip).xDD4_itemVar.pikachujoltground.xDE0 = 1;
        }
        k if k == It_Kind_Kirby_PikachuTJolt_Ground || k == It_Kind_Kirby_PichuTJolt_Ground => {
            ext::efSync_Spawn(0x4AD, gobj, jobj);
            (*ip).xDD4_itemVar.pikachujoltground.xDE0 = 1;
        }
        _ => {}
    }
    ext::db_80225DD8(gobj, owner);
}

pub unsafe extern "C" fn itPikachutjoltground_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.pikachujoltground.xDE8 = (*ip).pos;
    ext::it_80273130(gobj) != 0
}

pub unsafe extern "C" fn itPikachutjoltground_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.pikachujoltground.xDDC.is_null() {
        return true;
    }
    let flag: bool = if !gobj.is_null() && !ip.is_null() {
        if (*ip).xDD4_itemVar.pikachujoltground.xDDC.is_null() {
            true
        } else {
            ext::it_802B3EFC((*ip).xDD4_itemVar.pikachujoltground.xDDC) != gobj
        }
    } else {
        false
    };
    if flag {
        return true;
    }
    if itPikachutjoltground_UnkMotion0_Anim(gobj) {
        return true;
    }
    ext::it_802B3F20((*ip).xDD4_itemVar.pikachujoltground.xDDC, &mut (*ip).pos);
    false
}

pub unsafe extern "C" fn itPikachutjoltground_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPikachutJoltGroundAttributes;
    (*ip).x40_vel.x = (*attr).x8 * cosf((*ip).xDD4_itemVar.pikachujoltground.xDD4);
    (*ip).x40_vel.y = (*attr).x8 * sinf((*ip).xDD4_itemVar.pikachujoltground.xDD4);
}

/// 2B37F0 (static)
pub unsafe extern "C" fn itPikachutjoltground_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    itResetVelocity(ip);
}

pub unsafe extern "C" fn itPikachutjoltground_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);

    (*ip).xDD4_itemVar.pikachujoltground.xE00 = (*ip).xDD4_itemVar.pikachujoltground.xDF4;
    if ext::it_8026E9A4(
        gobj,
        &mut (*ip).xDD4_itemVar.pikachujoltground.xDE8,
        &mut (*ip).pos,
        &mut (*ip).xDD4_itemVar.pikachujoltground.xDF4,
    ) != 0
    {
        let mut ip2 = GET_ITEM(gobj);
        ext::efLib_DestroyAll(gobj);
        (*ip2).xDD4_itemVar.pikachujoltground.xDE0 = 0;
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
        ip2 = GET_ITEM(gobj);
        let ground_angle = ext::atan2f(
            (*ip2).xDD4_itemVar.pikachujoltground.xDF4.y,
            (*ip2).xDD4_itemVar.pikachujoltground.xDF4.x,
        );
        let mut angle_diff: f32 = ext::atan2f((*ip2).x40_vel.y, (*ip2).x40_vel.x) - ground_angle;

        while angle_diff < 0.0 {
            angle_diff = (angle_diff as f64 + M_TAU) as f32;
        }
        while angle_diff as f64 > M_TAU {
            angle_diff = (angle_diff as f64 - M_TAU) as f32;
        }
        let facing_dir: f32 = if (angle_diff as f64) < M_PI { -1.0 } else { 1.0 };
        (*ip2).xDD4_itemVar.pikachujoltground.xDDC =
            ext::it_802B4224((*ip2).owner, gobj, &mut (*ip2).pos, (*ip2).kind.wrapping_add(1), facing_dir);
        (*ip).xDD4_itemVar.pikachujoltground.xE00 = (*ip).xDD4_itemVar.pikachujoltground.xDF4;
        if (*ip).xDD4_itemVar.pikachujoltground.xDDC.is_null() {
            return true;
        }
        ext::it_802B3F88(
            (*ip).xDD4_itemVar.pikachujoltground.xDDC,
            &mut (*ip).pos,
            &mut (*ip).x378_itemColl,
            &mut (*ip).xDD4_itemVar.pikachujoltground.xDF4,
        );
        (*ip).xDD4_itemVar.pikachujoltground.xDE4 = 0;
        ext::it_802725D4(gobj);
    }
    false
}

pub unsafe extern "C" fn itPikachutjoltground_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);

    if (*ip).xDD4_itemVar.pikachujoltground.xDE4 >= 0 {
        (*ip).xDD4_itemVar.pikachujoltground.xE00 = (*ip).xDD4_itemVar.pikachujoltground.xDF4;
        if ext::it_8026E9A4(
            gobj,
            &mut (*ip).xDD4_itemVar.pikachujoltground.xDE8,
            &mut (*ip).pos,
            &mut (*ip).xDD4_itemVar.pikachujoltground.xDF4,
        ) != 0
        {
            let ip2 = GET_ITEM(gobj);
            let attr = (*(*ip2).xC4_article_data).x4_specialAttributes as *mut itPikachutJoltGroundAttributes;
            let flag: bool;

            if 0.0 == (*attr).xC {
                flag = false;
            } else {
                let angle = ext::lbVector_Angle(
                    &mut (*ip2).xDD4_itemVar.pikachujoltground.xDF4,
                    &mut (*ip2).xDD4_itemVar.pikachujoltground.xE00,
                );
                let mut limit = (*attr).xC;
                if limit < 0.0 {
                    limit = -limit;
                }
                flag = angle > limit;
            }

            if flag {
                return true;
            }

            if (*ip).xDD4_itemVar.pikachujoltground.xDDC.is_null() {
                return true;
            }
            ext::it_802B3F88(
                (*ip).xDD4_itemVar.pikachujoltground.xDDC,
                &mut (*ip).pos,
                &mut (*ip).x378_itemColl,
                &mut (*ip).xDD4_itemVar.pikachujoltground.xDF4,
            );
            (*ip).xDD4_itemVar.pikachujoltground.xDE4 = 0;
        }
    }
    (*ip).xDD4_itemVar.pikachujoltground.xDE4 = (*ip).xDD4_itemVar.pikachujoltground.xDE4.wrapping_add(1);
    false
}

pub unsafe extern "C" fn it_2725_Logic106_DmgDealt(gobj: *mut Item_GObj) -> bool {
    unlink_and_destroy(gobj);
    true
}

pub unsafe extern "C" fn it_2725_Logic106_Reflected(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);

    (*ip).facing_dir = -(*ip).facing_dir;
    HSD_JObjSetRotationY(jobj, (M_PI_2 * (*ip).facing_dir as f64) as f32);
    (*ip).xDD4_itemVar.pikachujoltground.xDD4 = ((*ip).xDD4_itemVar.pikachujoltground.xDD4 as f64 + M_PI) as f32;

    while (*ip).xDD4_itemVar.pikachujoltground.xDD4 < 0.0 {
        (*ip).xDD4_itemVar.pikachujoltground.xDD4 = ((*ip).xDD4_itemVar.pikachujoltground.xDD4 as f64 + M_TAU) as f32;
    }
    while (*ip).xDD4_itemVar.pikachujoltground.xDD4 as f64 > M_TAU {
        (*ip).xDD4_itemVar.pikachujoltground.xDD4 = ((*ip).xDD4_itemVar.pikachujoltground.xDD4 as f64 - M_TAU) as f32;
    }
    false
}

pub unsafe extern "C" fn it_2725_Logic106_Clanked(gobj: *mut Item_GObj) -> bool {
    unlink_and_destroy(gobj);
    true
}

pub unsafe extern "C" fn it_2725_Logic106_Absorbed(gobj: *mut Item_GObj) -> bool {
    unlink_and_destroy(gobj);
    true
}

pub unsafe extern "C" fn it_2725_Logic106_HitShield(gobj: *mut Item_GObj) -> bool {
    unlink_and_destroy(gobj);
    true
}

pub unsafe extern "C" fn it_2725_Logic106_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);

    ext::lbVector_Mirror(&mut (*ip).x40_vel, &mut (*ip).xC58);
    (*ip).xDD4_itemVar.pikachujoltground.xDD4 = ext::atan2f((*ip).x40_vel.y, (*ip).x40_vel.x);

    while (*ip).xDD4_itemVar.pikachujoltground.xDD4 < 0.0 {
        (*ip).xDD4_itemVar.pikachujoltground.xDD4 = ((*ip).xDD4_itemVar.pikachujoltground.xDD4 as f64 + M_TAU) as f32;
    }
    while (*ip).xDD4_itemVar.pikachujoltground.xDD4 as f64 > M_TAU {
        (*ip).xDD4_itemVar.pikachujoltground.xDD4 = ((*ip).xDD4_itemVar.pikachujoltground.xDD4 as f64 - M_TAU) as f32;
    }

    let dir: f32 = if (*ip).x40_vel.x >= 0.0 { 1.0 } else { -1.0 };
    (*ip).facing_dir = dir;
    HSD_JObjSetRotationY(jobj, (M_PI_2 * (*ip).facing_dir as f64) as f32);
    false
}

pub unsafe extern "C" fn itPikachuTJoltGround_Logic106_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
