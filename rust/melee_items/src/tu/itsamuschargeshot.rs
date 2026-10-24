//! Port of src/melee/it/kinds/itsamuschargeshot.c: Samus's (and Kirby's copy) charge shot (logic table 108).
//! Callees of unported TUs go through `ext` (unset: panics "TODO"). `sinf` / `cosf` are the real gekko ports
//! (`melee_runtime::gekko`). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itsamuschargeshot.rs.
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_MtxGetRotation(a0: *mut c_void, a1: *mut Vec) => unsafe { *(a1 as *mut _) = melee_runtime::tu::mtx::HSD_MtxGetRotation(&*(a0 as *const _)) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn efLib_DestroyAll(a0: *mut HSD_GObj);
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj) -> *mut c_void;
    fn ftKb_SpecialNSs_800FCC14(a0: *mut HSD_GObj, a1: *mut i32, a2: *mut i32) -> i32;
    fn ftKb_SpecialNSs_800FCC6C(a0: *mut HSD_GObj) -> i32;
    fn ftKb_SpecialNSs_800FCCBC(a0: *mut HSD_GObj) -> i32;
    fn ftKb_SpecialNSs_800FCD04(a0: *mut HSD_GObj);
    fn ftLib_800865C0(a0: *mut HSD_GObj) -> f32 => unsafe { crate::tu::ftlib::ftLib_800865C0(a0) };
    fn ftLib_80086630(a0: *mut HSD_GObj, a1: u32) -> *mut HSD_JObj => unsafe { crate::tu::ftlib::ftLib_80086630(a0, a1) };
    fn ftLib_800869D4(a0: *mut HSD_GObj) -> f32;
    fn ftSs_SpecialLw_80129100(a0: *mut HSD_GObj, a1: *mut i32, a2: *mut i32) -> i32;
    fn ftSs_SpecialLw_80129158(a0: *mut HSD_GObj) -> i32;
    fn ftSs_SpecialN_801291A8(a0: *mut HSD_GObj) -> i32;
    fn ftSs_SpecialN_801291F0(a0: *mut HSD_GObj);
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_80273130(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273130(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_8027429C(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_2725::it_8027429C(a0, a1 as _) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
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

/// `ItemStateTable it_803F7288[]`
pub static mut it_803F7288: [ItemStateTable; 9] = [
    st_row(0, pred_b(itSamuschargeshot_UnkMotion0_Anim), Some(itSamuschargeshot_UnkMotion0_Phys), pred_b(itSamuschargeshot_UnkMotion0_Coll)),
    st_row(1, pred_b(itSamuschargeshot_UnkMotion8_Anim), Some(itSamuschargeshot_UnkMotion8_Phys), pred_b(itSamuschargeshot_UnkMotion8_Coll)),
    st_row(2, pred_b(itSamuschargeshot_UnkMotion8_Anim), Some(itSamuschargeshot_UnkMotion8_Phys), pred_b(itSamuschargeshot_UnkMotion8_Coll)),
    st_row(3, pred_b(itSamuschargeshot_UnkMotion8_Anim), Some(itSamuschargeshot_UnkMotion8_Phys), pred_b(itSamuschargeshot_UnkMotion8_Coll)),
    st_row(4, pred_b(itSamuschargeshot_UnkMotion8_Anim), Some(itSamuschargeshot_UnkMotion8_Phys), pred_b(itSamuschargeshot_UnkMotion8_Coll)),
    st_row(5, pred_b(itSamuschargeshot_UnkMotion8_Anim), Some(itSamuschargeshot_UnkMotion8_Phys), pred_b(itSamuschargeshot_UnkMotion8_Coll)),
    st_row(6, pred_b(itSamuschargeshot_UnkMotion8_Anim), Some(itSamuschargeshot_UnkMotion8_Phys), pred_b(itSamuschargeshot_UnkMotion8_Coll)),
    st_row(7, pred_b(itSamuschargeshot_UnkMotion8_Anim), Some(itSamuschargeshot_UnkMotion8_Phys), pred_b(itSamuschargeshot_UnkMotion8_Coll)),
    st_row(8, pred_b(itSamuschargeshot_UnkMotion8_Anim), Some(itSamuschargeshot_UnkMotion8_Phys), pred_b(itSamuschargeshot_UnkMotion8_Coll)),
];

/// `Item_InitSpawnPositionFromParent` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitSpawnPositionFromParent(spawn: *mut SpawnItem, parent: *mut HSD_GObj, pos: *mut Vec3) {
    (*spawn).prev_pos = *pos;
    (*spawn).prev_pos.z = 0.0;
    ext::it_8026BB68(parent, &mut (*spawn).pos);
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
    (*spawn).x44_flag.anon_0.set_b0(initial_collision as _);
    (*spawn).x40 = 0;
}

/// `Item_ClearCmdVars` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_ClearCmdVars(ip: *mut Item) {
    (*ip).xDB8_itcmd_var3 = 0;
    (*ip).xDB4_itcmd_var2 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDAC_itcmd_var0 = 0;
}

/// `itGetJObjGrandchild` (it/inlines.h, static inline)
#[inline]
unsafe fn itGetJObjGrandchild(gobj: *mut Item_GObj) -> *mut HSD_JObj {
    let jobj = GET_JOBJ(gobj);
    let mut child = if jobj.is_null() { null_mut() } else { (*jobj).child };
    child = if child.is_null() { null_mut() } else { (*child).child };
    child
}

pub unsafe extern "C" fn it_802B5518(gobj: *mut Item_GObj, cd: *mut CollData) -> bool {
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

pub unsafe extern "C" fn it_802B55C8(gobj: *mut Fighter_GObj, pos: *mut Vec3, arg2: u32, arg3: i32, facing_dir: f32) -> *mut HSD_GObj {
    let mut si: SpawnItem = core::mem::zeroed();

    si.kind = arg3 as _;
    Item_InitSpawnPositionFromParent(&mut si, gobj, pos);
    Item_InitSpawnCommonFields(&mut si, gobj, facing_dir, true);
    let n = ext::Item_80268B18(&mut si);
    if !n.is_null() {
        let ip = GET_ITEM(n);
        let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusChargeShot_Attributes;
        Item_ClearCmdVars(ip);
        ext::it_80275158(n, (*attr).lifetime);
        (*ip).xDD4_itemVar.samuschargeshot.xDE8 = 0;
        (*ip).xDD4_itemVar.samuschargeshot.xDE0 = 0.0;
        (*ip).xDD4_itemVar.samuschargeshot.xDFC = 0;
        (*ip).xDD4_itemVar.samuschargeshot.xE00 = gobj;
        (*ip).xDD4_itemVar.samuschargeshot.xE04 = ext::ftLib_800869D4(gobj) as _;
        (*ip).xDD4_itemVar.samuschargeshot.xE2C = 0.0;
        (*ip).xDD4_itemVar.samuschargeshot.xE30 = 0.0;
        (*ip).xDD4_itemVar.samuschargeshot.xE34 = 0.0;
        ext::Item_8026AB54(n, gobj, arg2);
    }
    n
}

pub unsafe extern "C" fn it_802B56E4(gobj: *mut Item_GObj, vec: *mut Vec3, farg0: f32, mut farg1: f32, farg2: f32) {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusChargeShot_Attributes;
    let sv = &mut (*ip).xDD4_itemVar.samuschargeshot;

    sv.xDD8 = farg0;
    ext::it_80275158(gobj, (*attr).lifetime);
    if farg1 < 0.0 {
        farg1 = 0.0;
    }
    if farg1 > farg2 {
        farg1 = farg2;
    }
    sv.xDEC = farg1 as i32;
    sv.xDF0 = farg2 as i32;
    if !sv.xE00.is_null() && (*ip).owner == sv.xE00 {
        it_802B5CBC(gobj, sv.xE00);
        {
            let mut sp2C: Vec3 = core::mem::zeroed();
            let mut sp20: Vec3 = core::mem::zeroed();

            let j = ext::ftLib_80086630(sv.xE00, (*ip).xDC4);
            ext::HSD_MtxGetRotation((*j).mtx.as_mut_ptr() as *mut c_void, &mut sp2C);
            sp20.z = 0.0;
            sp20.y = 0.0;
            sp20.x = 0.0;
            ext::it_8027429C(gobj, &mut sp20);
        }
        (*ip).xDC8_word.flags.set_x14(0);
        ext::it_8026B3A8(gobj);
        // (farg1 * ((xC - x8) / farg2)) + x8: contracted
        sv.xDDC = fmadds(farg1, ((*attr).xC - (*attr).x8) / farg2, (*attr).x8);
        sv.xDF8 = fmadds(farg1, ((*attr).x14 - (*attr).x10) / farg2, (*attr).x10) as u32 as i32;
        sv.xDE0 = 0.0;
        sv.xDE4 = fmadds(sv.xDEC as f32, ((*attr).x1C - (*attr).x18) / (sv.xDF0 as f32), (*attr).x18);
        sv.xDF4 = 0;
        (*ip).facing_dir = ext::ftLib_800865C0(sv.xE00);
        (*ip).pos = *vec;
        sv.xDFC = 0;
        (*ip).x40_vel.x = sv.xDDC * cosf(sv.xDD8);
        (*ip).x40_vel.y = sv.xDDC * sinf(sv.xDD8);
        (*ip).x40_vel.z = 0.0;
        sv.xDE8 = 1;
    }
}

pub unsafe extern "C" fn it_2725_Logic108_Destroyed(gobj: *mut Item_GObj) {
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        ext::efLib_DestroyAll(gobj);
        if (*ip).xDD4_itemVar.samuschargeshot.xDE8 == 0 {
            if !(*ip).xDD4_itemVar.samuschargeshot.xE00.is_null() && (*ip).owner == (*ip).xDD4_itemVar.samuschargeshot.xE00 {
                let temp_r0 = (*ip).kind;
                match temp_r0 {
                    It_Kind_Samus_Charge => ext::ftSs_SpecialN_801291F0((*ip).xDD4_itemVar.samuschargeshot.xE00),
                    It_Kind_Kirby_SamusCharge => ext::ftKb_SpecialNSs_800FCD04((*ip).xDD4_itemVar.samuschargeshot.xE00),
                    _ => {}
                }
            }
            (*ip).xDD4_itemVar.samuschargeshot.xE00 = null_mut();
        }
        (*ip).owner = null_mut();
        (*ip).xDC8_word.flags.set_x13(0);
    }
}

pub unsafe extern "C" fn it_802B5974(gobj: *mut Item_GObj) {
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        ext::efLib_DestroyAll(gobj);
        (*ip).xDD4_itemVar.samuschargeshot.xDFC = 0;
        ext::Item_8026A8EC(gobj);
    }
}

pub unsafe extern "C" fn it_2725_Logic108_PickedUp(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let grandchild = itGetJObjGrandchild(gobj);

    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    match (*ip).kind {
        It_Kind_Samus_Charge => {
            ext::efSync_Spawn(0x47F, gobj, grandchild);
            (*ip).xDD4_itemVar.samuschargeshot.xDFC = 1;
        }
        It_Kind_Kirby_SamusCharge => {
            ext::efSync_Spawn(0x4A1, gobj, grandchild);
            (*ip).xDD4_itemVar.samuschargeshot.xDFC = 1;
        }
        _ => {}
    }
}

pub unsafe extern "C" fn itSamuschargeshot_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    let mut scale: Vec3 = core::mem::zeroed();
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusChargeShot_Attributes;
    let grandchild = itGetJObjGrandchild(gobj);
    let sv = &mut (*ip).xDD4_itemVar.samuschargeshot;

    if !sv.xE00.is_null() && (*ip).owner == sv.xE00 {
        match (*ip).kind {
            It_Kind_Samus_Charge => {
                if ext::ftSs_SpecialLw_80129158(sv.xE00) != 0 {
                    return true;
                }
                if ext::ftSs_SpecialN_801291A8(sv.xE00) != 0 {
                    return true;
                }
                if ext::ftSs_SpecialLw_80129100(sv.xE00, &mut sv.xDEC, &mut sv.xDF0) == -1 {
                    return true;
                }
            }
            It_Kind_Kirby_SamusCharge => {
                if ext::ftKb_SpecialNSs_800FCC6C(sv.xE00) != 0 {
                    return true;
                }
                if ext::ftKb_SpecialNSs_800FCCBC(sv.xE00) != 0 {
                    return true;
                }
                if ext::ftKb_SpecialNSs_800FCC14(sv.xE00, &mut sv.xDEC, &mut sv.xDF0) == -1 {
                    return true;
                }
            }
            _ => {}
        }
    }

    let s = fmadds(sv.xDEC as f32, ((*attr).x1C - (*attr).x18) / (sv.xDF0 as f32), (*attr).x18);
    scale.z = s;
    scale.y = s;
    scale.x = s;
    HSD_JObjSetScale(grandchild, &mut scale);
    false
}

pub unsafe extern "C" fn itSamuschargeshot_UnkMotion0_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itSamuschargeshot_UnkMotion0_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn it_802B5CBC(gobj: *mut Item_GObj, _unused: *mut Fighter_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusChargeShot_Attributes;

    ext::it_80275158(gobj, (*attr).lifetime);
    ext::efLib_DestroyAll(gobj);
    (*ip).xDD4_itemVar.samuschargeshot.xDFC = 0;

    if (*ip).xDD4_itemVar.samuschargeshot.xDEC < 0 {
        (*ip).xDD4_itemVar.samuschargeshot.xDEC = 0;
    }
    if (*ip).xDD4_itemVar.samuschargeshot.xDEC >= 8 {
        (*ip).xDD4_itemVar.samuschargeshot.xDEC = 7;
    }
    ext::Item_80268E5C(gobj, (*ip).xDD4_itemVar.samuschargeshot.xDEC + 1, ITEM_ANIM_UPDATE);
    (*ip).on_accessory = Some(it_802B5EDC);
}

pub unsafe extern "C" fn itSamuschargeshot_UnkMotion8_Anim(gobj: *mut Item_GObj) -> bool {
    let mut scale: Vec3 = core::mem::zeroed();
    let ip = (*gobj).user_data as *mut Item;
    let grandchild = itGetJObjGrandchild(gobj);

    scale.z = (*ip).xDD4_itemVar.samuschargeshot.xDE4;
    scale.y = scale.z;
    scale.x = scale.y;

    HSD_JObjSetScale(grandchild, &mut scale);
    ext::it_80273130(gobj) != 0
}

pub unsafe extern "C" fn itSamuschargeshot_UnkMotion8_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.x = (*ip).xDD4_itemVar.samuschargeshot.xDDC * cosf((*ip).xDD4_itemVar.samuschargeshot.xDD8);
    (*ip).x40_vel.y = (*ip).xDD4_itemVar.samuschargeshot.xDDC * sinf((*ip).xDD4_itemVar.samuschargeshot.xDD8);
}

pub unsafe extern "C" fn itSamuschargeshot_UnkMotion8_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if it_802B5518(gobj, &mut (*ip).x378_itemColl) {
        return true;
    }
    false
}

pub unsafe extern "C" fn it_802B5EDC(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);

    if (*ip).xDD4_itemVar.samuschargeshot.xDEC == (*ip).xDD4_itemVar.samuschargeshot.xDF0
        && (*ip).xDD4_itemVar.samuschargeshot.xDF4 == 0
    {
        match (*ip).kind {
            It_Kind_Samus_Charge => {
                ext::efSync_Spawn(0x480, gobj, jobj);
                ext::efSync_Spawn(0x481, gobj, jobj);
            }
            It_Kind_Kirby_SamusCharge => {
                ext::efSync_Spawn(0x4A2, gobj, jobj);
                ext::efSync_Spawn(0x4A3, gobj, jobj);
            }
            _ => {}
        }
    }
    (*ip).xDD4_itemVar.samuschargeshot.xDF4 = (*ip).xDD4_itemVar.samuschargeshot.xDF4.wrapping_add(1);
    (*ip).xDD4_itemVar.samuschargeshot.xDF4 = (*ip).xDD4_itemVar.samuschargeshot.xDF4 % 3;
}

pub unsafe extern "C" fn itSamusChargeshot_Logic108_DmgDealt(_arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itSamusChargeshot_Logic108_Clanked(_arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itSamusChargeshot_Logic108_Absorbed(_arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn it_2725_Logic108_Reflected(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);

    (*ip).facing_dir = -(*ip).facing_dir;
    let pi_over_two_x_facing_dir = (M_PI_2 * ((*ip).facing_dir as f64)) as f32;
    HSD_JObjSetRotationY(jobj, pi_over_two_x_facing_dir);

    let sv = &mut (*ip).xDD4_itemVar.samuschargeshot;
    sv.xDD8 = (sv.xDD8 as f64 + M_PI) as f32;
    while sv.xDD8 < 0.0 {
        sv.xDD8 = (sv.xDD8 as f64 + M_TAU) as f32;
    }
    while sv.xDD8 as f64 > M_TAU {
        sv.xDD8 = (sv.xDD8 as f64 - M_TAU) as f32;
    }
    false
}

pub unsafe extern "C" fn itSamusChargeshot_Logic108_HitShield(_arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn it_2725_Logic108_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);

    ext::lbVector_Mirror(&mut (*ip).x40_vel, &mut (*ip).xC58);
    let sv = &mut (*ip).xDD4_itemVar.samuschargeshot;
    sv.xDD8 = ext::atan2f((*ip).x40_vel.y, (*ip).x40_vel.x);
    while sv.xDD8 < 0.0 {
        sv.xDD8 = (sv.xDD8 as f64 + M_TAU) as f32;
    }
    while sv.xDD8 as f64 > M_TAU {
        sv.xDD8 = (sv.xDD8 as f64 - M_TAU) as f32;
    }

    (*ip).facing_dir = if (*ip).x40_vel.x >= 0.0 { 1.0 } else { -1.0 };
    let pi_over_two_x_facing_dir = (M_PI_2 * ((*ip).facing_dir as f64)) as f32;
    HSD_JObjSetRotationY(jobj, pi_over_two_x_facing_dir);
    false
}

pub unsafe extern "C" fn itSamusChargeshot_Logic108_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
