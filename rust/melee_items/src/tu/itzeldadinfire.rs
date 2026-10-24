//! Port of src/melee/it/kinds/itzeldadinfire.c: Zelda's Din's Fire ball (logic table 65). Callees of unported TUs
//! go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itzeldadinfire.rs.
//!
//! Fused sites (checked in the IR): the scale `xDD8 * ((xC - x8) / x4) + x8` (Anim 0 and 1) and the steering
//! `facing_dir * (x24 * f1) + xDE8` (Phys 0) are `fmadds`. `sinf` / `cosf` are the `melee_runtime::gekko` ports.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn efLib_DestroyAll(a0: *mut HSD_GObj);
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj) -> *mut c_void;
    fn ftLib_800865D8(a0: *mut HSD_GObj, a1: *mut f32, a2: *mut f32);
    fn ftZd_SpecialLw_8013B540(a0: *mut HSD_GObj) -> i32;
    fn ftZd_SpecialLw_8013B574(a0: *mut HSD_GObj) -> i32;
    fn ftZd_SpecialLw_8013B5C4(a0: *mut HSD_GObj);
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_802725D4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802725D4(a0) };
    fn it_80272940(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272940(a0) };
    fn it_80273130(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273130(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32);
    fn it_802C4580(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: *mut Vec, a3: f32, a4: f32) -> *mut HSD_GObj => unsafe { crate::tu::itzeldadinfireexplode::it_802C4580(a0 as _, a1, a2 as _, a3, a4) as _ };
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

/// `ItemStateTable ItemStateTable_ZeldaDinFire[]`
pub static mut ItemStateTable_ZeldaDinFire: [ItemStateTable; 2] = [
    st_row(0, pred_b(itZeldadinfire_UnkMotion0_Anim), Some(itZeldadinfire_UnkMotion0_Phys), pred_b(itZeldadinfire_UnkMotion0_Coll)),
    st_row(1, pred_b(itZeldadinfire_UnkMotion1_Anim), Some(itZeldadinfire_UnkMotion1_Phys), pred_b(itZeldadinfire_UnkMotion1_Coll)),
];

/// The file-local `ItZeldaDinFire_ItemVars` (the article's special attributes).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ItZeldaDinFire_ItemVars {
    pub x0: f32,
    pub x4: f32,
    pub x8: f32,
    pub xC: f32,
    pub x10: f32,
    pub x14: f32,
    pub x18: f32,
    pub x1C: f32,
    pub x20: f32,
    pub x24: f32,
    pub x28: f32,
    pub x2C: f32,
}

#[inline]
fn abs_f(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

pub unsafe extern "C" fn itZeldaDinFire_GetOwner(gobj: *mut Item_GObj) -> *mut Fighter_GObj {
    (*GET_ITEM(gobj)).owner
}

pub unsafe extern "C" fn it_802C3AFC(gobj: *mut Item_GObj, cd: *mut CollData) -> bool {
    let ip = GET_ITEM(gobj);
    let mut ret = false;
    ext::it_8026DA08(gobj);
    if (*ip).x40_vel.y > 0.0 {
        if (*cd).env_flags & (Collide_CeilingPush | Collide_CeilingHug) != 0 {
            ret = true;
        }
    } else if (*cd).env_flags & (Collide_FloorPush | Collide_FloorHug) != 0 {
        ret = true;
    }
    if (*ip).x40_vel.x > 0.0 {
        if (*cd).env_flags & Collide_LeftWallMask != 0 {
            ret = true;
        }
    } else if (*cd).env_flags & Collide_RightWallMask != 0 {
        ret = true;
    }
    ret
}

pub unsafe extern "C" fn it_802C3BAC(gobj: *mut Item_GObj, vec: *mut Vec, facing_dir: f32, arg3: f32) -> *mut Item_GObj {
    let mut si: SpawnItem = core::mem::zeroed();
    si.kind = It_Kind_Zelda_DinFire;
    // Item_InitSpawnPositionFromParent(&si, gobj, vec)
    si.prev_pos = *vec;
    si.prev_pos.z = 0.0;
    ext::it_8026BB68(gobj, &mut si.pos);
    // Item_InitSpawnCommonFields(&si, gobj, facing_dir, true)
    si.facing_dir = facing_dir;
    si.x3C_damage = 0;
    si.vel.z = 0.0;
    si.vel.y = 0.0;
    si.vel.x = 0.0;
    si.x0_parent_gobj = gobj;
    si.x4_parent_gobj2 = si.x0_parent_gobj;
    si.x44_flag.anon_0.set_b0(1);
    si.x40 = 0;
    let n = ext::Item_80268B18(&mut si);
    if !n.is_null() {
        let i = GET_ITEM(n);
        let attrs = (*(*i).xC4_article_data).x4_specialAttributes as *mut ItZeldaDinFire_ItemVars;
        // Item_ClearCmdVars(i)
        (*i).xDB8_itcmd_var3 = 0;
        (*i).xDB4_itcmd_var2 = 0;
        (*i).xDB0_itcmd_var1 = 0;
        (*i).xDAC_itcmd_var0 = 0;
        ext::it_80275158(n, (*attrs).x0);
        (*i).xDD4_itemVar.zeldadinfire.xDD8 = 0.0;
        (*i).xDD4_itemVar.zeldadinfire.xDDC = 0;
        (*i).xDD4_itemVar.zeldadinfire.xDE0 = gobj;
        (*i).xDD4_itemVar.zeldadinfire.xDE4 = 0;
        it_802C3D74(n);
    }
    n
}

pub unsafe extern "C" fn itZeldaDinFire_Logic65_Destroyed(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_802725D4(gobj);
    if !(*ip).xDD4_itemVar.zeldadinfire.xDE0.is_null() {
        if (*ip).owner == (*ip).xDD4_itemVar.zeldadinfire.xDE0 {
            ext::ftZd_SpecialLw_8013B5C4((*ip).xDD4_itemVar.zeldadinfire.xDE0);
        }
        (*ip).xDD4_itemVar.zeldadinfire.xDE0 = null_mut();
    }
    if (*ip).xDD4_itemVar.zeldadinfire.xDF4 != 0 {
        ext::efLib_DestroyAll(gobj);
        (*ip).xDD4_itemVar.zeldadinfire.xDF4 = 0;
    }
    (*ip).xDC8_word.flags.set_x13(0);
}

pub unsafe extern "C" fn it_802C3D44(gobj: *mut Item_GObj) {
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        if !ip.is_null() {
            (*ip).owner = null_mut();
            (*ip).xDD4_itemVar.zeldadinfire.xDE0 = null_mut();
            (*ip).xDC8_word.flags.set_x13(0);
        }
    }
}

pub unsafe extern "C" fn it_802C3D74(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut ItZeldaDinFire_ItemVars;
    // Item_ClearFlagsAndEnterState(gobj, ip, 0)
    ext::it_8026B3A8(gobj);
    (*ip).xDC8_word.flags.set_x13(0);
    ext::it_80272940(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    ext::it_80275158(gobj, (*attrs).x0);
    (*ip).xDD4_itemVar.zeldadinfire.xDD8 = 0.0;
    (*ip).xDD4_itemVar.zeldadinfire.xDDC = 0;
    (*ip).xDD4_itemVar.zeldadinfire.xDE8 = (*attrs).x10 * (*ip).facing_dir;
    let rot: f64 = if 1.0 == (*ip).facing_dir { 0.0 } else { core::f64::consts::PI };
    (*ip).xDD4_itemVar.zeldadinfire.xDEC = rot as f32;
    (*ip).xDD4_itemVar.zeldadinfire.xDF0 = (*attrs).x14;
    ext::efSync_Spawn(1272, gobj, jobj);
    (*ip).xDD4_itemVar.zeldadinfire.xDF4 = 1;
    (*ip).x40_vel.x = (*ip).xDD4_itemVar.zeldadinfire.xDF0
        * cosf((*ip).xDD4_itemVar.zeldadinfire.xDEC + (*ip).xDD4_itemVar.zeldadinfire.xDE8);
    (*ip).x40_vel.y = (*ip).xDD4_itemVar.zeldadinfire.xDF0
        * sinf((*ip).xDD4_itemVar.zeldadinfire.xDEC + (*ip).xDD4_itemVar.zeldadinfire.xDE8);
    (*ip).x40_vel.z = 0.0;
    ext::db_80225DD8(gobj, (*ip).owner);
}

/// `itZeldadinfire_UnkMotion0_Anim_inline` (static inline)
#[inline]
unsafe fn itZeldadinfire_UnkMotion0_Anim_inline(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut ItZeldaDinFire_ItemVars;
    let jobj = GET_JOBJ(gobj);
    if (*ip).xDD4_itemVar.zeldadinfire.xDF4 != 0 {
        ext::efLib_DestroyAll(gobj);
        (*ip).xDD4_itemVar.zeldadinfire.xDF4 = 0;
    }
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    ext::efSync_Spawn(1273, gobj, jobj);
    (*ip).xDD4_itemVar.zeldadinfire.xDF4 = 1;
    (*ip).xD44_lifeTimer = (*attrs).x2C;
}

pub unsafe extern "C" fn itZeldadinfire_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut ItZeldaDinFire_ItemVars;
    let mut v: Vec = core::mem::zeroed();
    if (*ip).xDD4_itemVar.zeldadinfire.xDDC == 0 {
        (*ip).xDD4_itemVar.zeldadinfire.xDD8 += 1.0;
        if (*ip).xDD4_itemVar.zeldadinfire.xDD8 >= (*attrs).x4 {
            (*ip).xDD4_itemVar.zeldadinfire.xDD8 = (*attrs).x4;
        }
        if !(*ip).xDD4_itemVar.zeldadinfire.xDE0.is_null()
            && (*ip).owner == (*ip).xDD4_itemVar.zeldadinfire.xDE0
            && ext::ftZd_SpecialLw_8013B574((*ip).xDD4_itemVar.zeldadinfire.xDE0) == 1
        {
            itZeldadinfire_UnkMotion0_Anim_inline(gobj);
        }
    }
    let s = fmadds((*ip).xDD4_itemVar.zeldadinfire.xDD8, ((*attrs).xC - (*attrs).x8) / (*attrs).x4, (*attrs).x8);
    v.z = s;
    v.y = s;
    v.x = s;
    HSD_JObjSetScale(jobj, &mut v);
    if ext::it_80273130(gobj) == 1 {
        itZeldadinfire_UnkMotion0_Anim_inline(gobj);
    }
    false
}

pub unsafe extern "C" fn itZeldadinfire_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut ItZeldaDinFire_ItemVars;
    let mut v: Vec = core::mem::zeroed();
    let s = fmadds((*ip).xDD4_itemVar.zeldadinfire.xDD8, ((*attrs).xC - (*attrs).x8) / (*attrs).x4, (*attrs).x8);
    v.z = s;
    v.y = s;
    v.x = s;
    HSD_JObjSetScale(jobj, &mut v);
    if ext::it_80273130(gobj) == 1 {
        if (*ip).xDD4_itemVar.zeldadinfire.xDF4 != 0 {
            ext::efLib_DestroyAll(gobj);
            (*ip).xDD4_itemVar.zeldadinfire.xDF4 = 0;
        }
        if !(*ip).xDD4_itemVar.zeldadinfire.xDE0.is_null() && (*ip).xDD4_itemVar.zeldadinfire.xDE0 == (*ip).owner {
            ext::it_802C4580(
                (*ip).owner,
                gobj,
                &mut (*ip).pos,
                (*ip).facing_dir,
                (*ip).xDD4_itemVar.zeldadinfire.xDD8,
            );
            ext::ftZd_SpecialLw_8013B5C4((*ip).xDD4_itemVar.zeldadinfire.xDE0);
        }
        return true;
    }
    false
}

pub unsafe extern "C" fn itZeldadinfire_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut ItZeldaDinFire_ItemVars;
    let mut f1: f32 = 0.0;
    let mut f2: f32 = 0.0;
    if (*ip).xDD4_itemVar.zeldadinfire.xDDC == 0 {
        if !(*ip).xDD4_itemVar.zeldadinfire.xDE0.is_null()
            && (*ip).owner == (*ip).xDD4_itemVar.zeldadinfire.xDE0
            && ext::ftZd_SpecialLw_8013B540((*ip).xDD4_itemVar.zeldadinfire.xDE0) == 1
        {
            ext::ftLib_800865D8((*ip).xDD4_itemVar.zeldadinfire.xDE0, &mut f2, &mut f1);
            if abs_f(f1) > (*attrs).x20 {
                (*ip).xDD4_itemVar.zeldadinfire.xDE8 =
                    fmadds((*ip).facing_dir, (*attrs).x24 * f1, (*ip).xDD4_itemVar.zeldadinfire.xDE8);
                if abs_f((*ip).xDD4_itemVar.zeldadinfire.xDE8) > (*attrs).x28 {
                    (*ip).xDD4_itemVar.zeldadinfire.xDE8 =
                        if (*ip).xDD4_itemVar.zeldadinfire.xDE8 > 0.0 { (*attrs).x28 } else { -(*attrs).x28 };
                }
            }
            (*ip).xDD4_itemVar.zeldadinfire.xDF0 += (*attrs).x18;
            if (*ip).xDD4_itemVar.zeldadinfire.xDF0 > (*attrs).x1C {
                (*ip).xDD4_itemVar.zeldadinfire.xDF0 = (*attrs).x1C;
            }
            (*ip).x40_vel.x = (*ip).xDD4_itemVar.zeldadinfire.xDF0
                * cosf((*ip).xDD4_itemVar.zeldadinfire.xDEC + (*ip).xDD4_itemVar.zeldadinfire.xDE8);
            (*ip).x40_vel.y = (*ip).xDD4_itemVar.zeldadinfire.xDF0
                * sinf((*ip).xDD4_itemVar.zeldadinfire.xDEC + (*ip).xDD4_itemVar.zeldadinfire.xDE8);
            (*ip).x40_vel.z = 0.0;
        }
    }
}

pub unsafe extern "C" fn itZeldadinfire_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    itResetVelocity(ip);
}

pub unsafe extern "C" fn itZeldadinfire_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if it_802C3AFC(gobj, &mut (*ip).x378_itemColl) {
        itZeldadinfire_UnkMotion0_Anim_inline(gobj);
    }
    false
}

pub unsafe extern "C" fn itZeldadinfire_UnkMotion1_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itZeldaDinFire_Logic65_Reflected(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    (*ip).xDD4_itemVar.zeldadinfire.xDDC = 1;
    (*ip).facing_dir = -(*ip).facing_dir;
    HSD_JObjSetRotationY(jobj, (core::f64::consts::FRAC_PI_2 * (*ip).facing_dir as f64) as f32);
    (*ip).x40_vel.x = -(*ip).x40_vel.x;
    (*ip).x40_vel.y = -(*ip).x40_vel.y;
    (*ip).x40_vel.z = 0.0;
    false
}

pub unsafe extern "C" fn itZeldaDinFire_Logic65_Clanked(_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itZeldaDinFire_Logic65_Absorbed(_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itZeldaDinFire_Logic65_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
