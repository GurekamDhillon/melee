//! src/melee/ft/ftlib.c: the fighter library (kind/part/position accessors, fighter-list walks, damage and
//! control helpers every other fighter, item and camera TU calls). Generated from the C by
//! `melee_world/gen/mtu/c2rs.py` (libclang AST: C integer / float semantics, clang's -ffp-contract=on fusion).
//! Regenerate with `bash rust/melee_fighter/gen/ftlib/regen.sh`. Calls out of the TU are `ext::` hooks
//! (linked by `melee_link`, unset: panic "TODO"). The hand port `crate::ftlib` (Rust-typed, oracle-checked by
//! tests/ftlib_ref.rs) stays for the legacy `vs_match`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use crate::types::*;


pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
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
    fn Camera_80030A50() -> *mut HSD_GObj;
    fn Camera_80031154(a0: *mut Vec) -> i32;
    fn Camera_LogicToScreen(a0: *mut CmSubject, a1: *mut S32Vec2) -> i32;
    fn Camera_RequestQuake(a0: CmQuakeKind, a1: *mut Vec);
    fn Fighter_ResetInputData_80068854(a0: *mut HSD_GObj);
    fn Fighter_UnkProcessDeath_80068354(a0: *mut HSD_GObj);
    fn Fighter_UpdateModelScale(a0: *mut HSD_GObj);
    fn Geno_DefineBaseKind(a0: i32) -> i32;
    fn Geno_DefineTemplateSkeleton(a0: i32) -> i32;
    fn HSD_PadRumbleRemoveId(a0: u8, a1: i32);
    fn HSD_Randi(a0: i32) -> i32;
    fn Mex_SsmForPortKind(a0: i32) -> i32;
    fn Player_800325C8(a0: CharacterKind, a1: i32) -> i8;
    fn Player_8003544C(a0: i32, a1: i32) -> i32;
    fn Player_GetMoreFlagsBit4(a0: i32) -> i32;
    fn Slippi_Codes() -> i32;
    fn atan2f(a0: f32, a1: f32) -> f32;
    fn efAsync_LoadSync(a0: i32);
    fn ftAnim_IsFramesRemaining(a0: *mut HSD_GObj) -> i32;
    fn ftCo_800C8348(a0: *mut HSD_GObj, a1: i32, a2: i32);
    fn ftCo_800D105C(a0: *mut HSD_GObj);
    fn ftCo_800D4F24(a0: *mut HSD_GObj, a1: i32);
    fn ftCommon_8007D92C(a0: *mut HSD_GObj);
    fn ftCommon_8007E6DC(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: i32);
    fn ftCommon_8007E82C(a0: *mut HSD_GObj);
    fn ftCommon_8007EBAC(a0: *mut Fighter, a1: u32, a2: u32);
    fn ftCommon_8007ED2C(a0: *mut Fighter);
    fn ftCommon_80080144(a0: *mut Fighter) -> i32;
    fn ftCommon_80080174(a0: *mut Fighter);
    fn ftCommon_GetModelScale(a0: *mut Fighter) -> f32;
    fn ftData_80085560(a0: i32, a1: i32);
    fn ftData_8008572C(a0: FighterKind);
    fn ftData_800857E0(a0: FighterKind);
    fn ftData_80085820(a0: FighterKind, a1: i32);
    fn ftData_80085A14(a0: FighterKind);
    fn ftGw_Init_8014A7F4(a0: *mut HSD_GObj, a1: *mut ItemModStruct);
    fn ftGw_Init_8014A814(a0: *mut HSD_GObj, a1: *mut ItemModStruct);
    fn ftKb_Init_800EEB00(a0: *mut HSD_GObj, a1: *mut *mut ArticleDynamicBones);
    fn ftKb_Init_800EEB1C(a0: *mut HSD_GObj, a1: *mut i32);
    fn ftKb_SpecialN_800EED50(a0: i32, a1: i32);
    fn ftParts_GetBoneIndex(a0: *mut Fighter, a1: Fighter_Part) -> Fighter_Part;
    fn ft_80081C88(a0: *mut HSD_GObj, a1: f32);
    fn ft_PlaySFX(a0: *mut Fighter, a1: i32, a2: u8, a3: u8);
    fn gm_8016B168() -> i32;
    fn itGetKind(a0: *mut HSD_GObj) -> ItemKind;
    fn it_8026B560(a0: *mut HSD_GObj) -> f32;
    fn it_8026B574(a0: *mut HSD_GObj) -> f32;
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec);
    fn lb_80014574(a0: u8, a1: i32, a2: i32, a3: i32);
    fn pl_8003E4A4(a0: i32, a1: i32, a2: *mut c_void, a3: i32);
    }
    pub use crate::tu::ftdata::CostumeListsForeachCharacter;
    pub use melee_runtime::tu::gobj_native::HSD_GObjPLinkHead;
    pub use melee_runtime::shared::ftData_UnkBytePerCharacter;
    pub use melee_runtime::shared::p_ftCommonData;
}



/// `ftLib_800860C4`
pub unsafe extern "C" fn ftLib_800860C4() -> i32 {
    melee_core::ct!("ftLib_800860C4");
    let mut ret: i32 = 0;
    let mut cur: *mut HSD_GObj = null_mut();
    {
        cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
        'l17: while !cur.is_null() {
            ret = ret + 1;
            cur = (*cur).next;
        }
    }
    return ret;
}

/// `ftLib_IsMasterHandPresent`
pub unsafe extern "C" fn ftLib_IsMasterHandPresent() -> i32 {
    melee_core::ct!("ftLib_IsMasterHandPresent");
    let mut __: [u8; 8] = [0; 8];
    let mut cur: *mut HSD_GObj = null_mut();
    {
        cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
        'l18: while !cur.is_null() {
            if ftLib_GetKind(cur) == ((Ft_Kind_MasterH as i32) as u32) {
                return 1;
            }
            cur = (*cur).next;
        }
    }
    return 0;
}

/// `ftLib_IsCrazyHandPresent`
pub unsafe extern "C" fn ftLib_IsCrazyHandPresent() -> i32 {
    melee_core::ct!("ftLib_IsCrazyHandPresent");
    let mut __: [u8; 8] = [0; 8];
    let mut cur: *mut HSD_GObj = null_mut();
    {
        cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
        'l19: while !cur.is_null() {
            if ftLib_GetKind(cur) == ((Ft_Kind_CrezyH as i32) as u32) {
                return 1;
            }
            cur = (*cur).next;
        }
    }
    return 0;
}

/// `ftLib_80086198`
pub unsafe extern "C" fn ftLib_80086198(mut gobj: *mut HSD_GObj) -> *mut HSD_GObj {
    melee_core::ct!("ftLib_80086198");
    let mut fp: *mut Fighter = null_mut();
    let mut cur: *mut HSD_GObj = null_mut();
    let mut result: *mut HSD_GObj = null_mut();
    let mut min_percent: i32 = 999;
    let mut __: [u8; 8] = [0; 8];
    if !gobj.is_null() {
        fp = HSD_GObjGetUserData(gobj) as *mut Fighter;
    } else {
        fp = null_mut();
    }
    result = null_mut();
    {
        cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
        'l20: while !cur.is_null() {
            'c20: {
                if !(ftLib_80086FD4(gobj, cur) != 0) {
                    let mut cur_fp: *mut Fighter = HSD_GObjGetUserData(cur) as *mut Fighter;
                    if (*cur_fp).x221F_b3() != 0 {
                        break 'c20;
                    }
                    if ((ext::gm_8016B168() != 0) && (!fp.is_null())) && (((*cur_fp).team as i32) == ((*fp).team as i32)) {
                        break 'c20;
                    }
                    if (*cur_fp).dmg.x1830_percent < (min_percent as f32) {
                        min_percent = (*cur_fp).dmg.x1830_percent as i32;
                        result = cur;
                    }
                }
            }
            cur = (*cur).next;
        }
    }
    return result;
}

/// `ftLib_8008627C`
pub unsafe extern "C" fn ftLib_8008627C(mut pos: *mut Vec, mut gobj: *mut HSD_GObj) -> *mut HSD_GObj {
    melee_core::ct!("ftLib_8008627C");
    let mut cur_v: Vec3 = unsafe { core::mem::zeroed() };
    let mut dist: f32 = 0.0;
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut cur_fp: *mut Fighter = null_mut();
    let mut fp: *mut Fighter = null_mut();
    let mut cur: *mut HSD_GObj = null_mut();
    let mut result: *mut HSD_GObj = null_mut();
    let mut __: [u8; 16] = [0; 16];
    let mut min_dist: f32 = f32::MAX;
    if !gobj.is_null() {
        fp = (*gobj).user_data as *mut Fighter;
    } else {
        fp = null_mut();
    }
    result = null_mut();
    {
        cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
        'l21: while !cur.is_null() {
            'c21: {
                if ftLib_80086FD4(gobj, cur) != 0 {
                    break 'c21;
                }
                cur_fp = (*cur).user_data as *mut Fighter;
                if (((*cur_fp).x221F_b3() as i32) != 0) || (((ext::gm_8016B168() != 0) && (!fp.is_null())) && (((*cur_fp).team as i32) == ((*fp).team as i32))) {
                    break 'c21;
                }
                ftLib_800866DC(cur, core::ptr::addr_of_mut!(cur_v) as *mut Vec);
                dx = (*pos).x - cur_v.x;
                dy = (*pos).y - cur_v.y;
                dist = fmadds(dx, dx, dy * dy);
                if dist < min_dist {
                    min_dist = dist;
                    result = cur;
                }
            }
            cur = (*cur).next;
        }
    }
    return result;
}

/// `ftLib_80086368`
pub unsafe extern "C" fn ftLib_80086368(mut v: *mut Vec, mut gobj: *mut HSD_GObj, mut facing_dir: f32) -> *mut HSD_GObj {
    melee_core::ct!("ftLib_80086368");
    let mut sp24: Vec3 = unsafe { core::mem::zeroed() };
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut diff: f32 = 0.0;
    let mut cur_fp: *mut Fighter = null_mut();
    let mut fp: *mut Fighter = null_mut();
    let mut cur: *mut HSD_GObj = null_mut();
    let mut result: *mut HSD_GObj = null_mut();
    let mut __: [u8; 16] = [0; 16];
    let mut min_diff: f32 = f32::MAX;
    if !gobj.is_null() {
        fp = (*gobj).user_data as *mut Fighter;
    } else {
        fp = null_mut();
    }
    result = null_mut();
    {
        cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
        'l22: while !cur.is_null() {
            'c22: {
                if ftLib_80086FD4(gobj, cur) != 0 {
                    break 'c22;
                }
                cur_fp = (*cur).user_data as *mut Fighter;
                if (((*cur_fp).x221F_b3() as i32) != 0) || (((ext::gm_8016B168() != 0) && (!fp.is_null())) && (((*cur_fp).team as i32) == ((*fp).team as i32))) {
                    break 'c22;
                }
                ftLib_800866DC(cur, core::ptr::addr_of_mut!(sp24) as *mut Vec);
                if ((facing_dir == ((-1_i32) as f32)) && (sp24.x > (*v).x)) || ((facing_dir == (1 as f32)) && (sp24.x < (*v).x)) {
                    break 'c22;
                }
                dx = (*v).x - sp24.x;
                dy = (*v).y - sp24.y;
                diff = fmadds(dx, dx, dy * dy);
                if diff < min_diff {
                    min_diff = diff;
                    result = cur;
                }
            }
            cur = (*cur).next;
        }
    }
    return result;
}

/// `sgn`
pub unsafe extern "C" fn sgn(mut x: f32) -> i32 {
    melee_core::ct!("sgn");
    if x < 0.0f32 {
        return (-1_i32);
    } else {
        return 1;
    }
}

/// `ftLib_800864A8`
pub unsafe extern "C" fn ftLib_800864A8(mut v: *mut Vec, mut gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftLib_800864A8");
    let mut vec: Vec3 = unsafe { core::mem::zeroed() };
    let mut fp: *mut Fighter = null_mut();
    let mut cur: *mut HSD_GObj = null_mut();
    let mut __: [u8; 12] = [0; 12];
    let mut result: i32 = 0;
    if !gobj.is_null() {
        fp = (*gobj).user_data as *mut Fighter;
    } else {
        fp = null_mut();
    }
    {
        cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
        'l23: while !cur.is_null() {
            'c23: {
                if ftLib_80086FD4(gobj, cur) != 0 {
                    break 'c23;
                }
                {
                    let mut cur_fp: *mut Fighter = (*cur).user_data as *mut Fighter;
                    if (((*cur_fp).x221F_b3() as i32) != 0) || (((ext::gm_8016B168() != 0) && (!fp.is_null())) && (((*cur_fp).team as i32) == ((*fp).team as i32))) {
                        break 'c23;
                    }
                    {
                        if (ext::Slippi_Codes() == 2) && ((*cur_fp).motion_id <= 11) {
                            break 'c23;
                        }
                    }
                    ftLib_800866DC(cur, core::ptr::addr_of_mut!(vec) as *mut Vec);
                    result = result + sgn(vec.x - (*v).x);
                }
            }
            cur = (*cur).next;
        }
    }
    if result == 0 {
        result = if ext::HSD_Randi(2) != 0 { 1 } else { (-1_i32) };
    }
    if result < 0 {
        return (-1_i32) as f32;
    } else {
        return 1 as f32;
    }
}

/// `ftLib_800865C0`
pub unsafe extern "C" fn ftLib_800865C0(mut gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftLib_800865C0");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).facing_dir;
}

/// `ftLib_800865CC`
pub unsafe extern "C" fn ftLib_800865CC(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_800865CC");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).ground_or_air as i32;
}

/// `ftLib_800865D8`
pub unsafe extern "C" fn ftLib_800865D8(mut gobj: *mut HSD_GObj, mut x: *mut f32, mut y: *mut f32) {
    melee_core::ct!("ftLib_800865D8");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    *x = (*fp).input.lstick[0 as usize].x;
    *y = (*fp).input.lstick[0 as usize].y;
}

/// `ftLib_800865F0`
pub unsafe extern "C" fn ftLib_800865F0(mut gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    melee_core::ct!("ftLib_800865F0");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*(*fp).parts.offset(ext::ftParts_GetBoneIndex(fp, 4 as u32) as isize)).joint;
}

/// `ftLib_80086630`
pub unsafe extern "C" fn ftLib_80086630(mut gobj: *mut HSD_GObj, mut part: Fighter_Part) -> *mut HSD_JObj {
    melee_core::ct!("ftLib_80086630");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*(*fp).parts.offset(part as isize)).joint;
}

/// `ftLib_80086644`
pub unsafe extern "C" fn ftLib_80086644(mut gobj: *mut HSD_GObj, mut pos: *mut Vec) {
    melee_core::ct!("ftLib_80086644");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    *pos = (*fp).cur_pos;
}

/// `ftLib_80086664`
pub unsafe extern "C" fn ftLib_80086664(mut gobj: *mut HSD_GObj, mut pos: *mut Vec) {
    melee_core::ct!("ftLib_80086664");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    (*fp).cur_pos = *pos;
}

/// `ftLib_80086684`
pub unsafe extern "C" fn ftLib_80086684(mut gobj: *mut HSD_GObj, mut pos: *mut Vec) {
    melee_core::ct!("ftLib_80086684");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    *pos = (*fp).prev_pos;
}

/// `ftLib_SetScale`
pub unsafe extern "C" fn ftLib_SetScale(mut gobj: *mut HSD_GObj, mut val: f32) {
    melee_core::ct!("ftLib_SetScale");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    (*fp).x34_scale.y = val;
    ext::Fighter_UpdateModelScale(gobj);
    ext::ftCommon_80080174(fp);
}

/// `ftLib_800866DC`
pub unsafe extern "C" fn ftLib_800866DC(mut gobj: *mut HSD_GObj, mut v: *mut Vec) {
    melee_core::ct!("ftLib_800866DC");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut r4: *mut ftCo_DatAttrs = core::ptr::addr_of_mut!((*fp).co_attrs) as *mut ftCo_DatAttrs;
    let mut i: i32 = (*(*(*fp).ft_data).x0).camera_zoom_target_bone;
    ext::lb_8000B1CC(ftLib_80086630(gobj, i as u32), core::ptr::addr_of_mut!((*r4).x170) as *mut Vec, v);
}

/// `ftLib_80086724`
pub unsafe extern "C" fn ftLib_80086724(mut gobj: *mut HSD_GObj, mut other: *mut HSD_GObj) {
    melee_core::ct!("ftLib_80086724");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if (!(*fp).item_gobj.is_null()) || (!(*fp).x1978.is_null()) {
        ext::ftCommon_8007E6DC(gobj, other, 1);
    }
}

/// `ftLib_80086764`
pub unsafe extern "C" fn ftLib_80086764(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLib_80086764");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if !(*fp).x1984_heldItemSpec.is_null() {
        ext::ftCommon_8007E82C(gobj);
    }
}

/// `ftLib_GetItem`
pub unsafe extern "C" fn ftLib_GetItem(mut gobj: *mut HSD_GObj) -> *mut HSD_GObj {
    melee_core::ct!("ftLib_GetItem");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).item_gobj;
}

/// `ftLib_800867A0`
pub unsafe extern "C" fn ftLib_800867A0(mut gobj: *mut HSD_GObj, mut arg1: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_800867A0");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if (ftLib_GetItem(gobj) == arg1) || ((*fp).x1978 == arg1) {
        return 1;
    } else {
        return 0;
    }
}

/// `ftLib_800867CC`
pub unsafe extern "C" fn ftLib_800867CC(mut gobj: *mut HSD_GObj) -> *mut HSD_GObj {
    melee_core::ct!("ftLib_800867CC");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).x1984_heldItemSpec;
}

/// `ftLib_800867D8`
pub unsafe extern "C" fn ftLib_800867D8(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_800867D8");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).x221D_b4() as i32;
}

/// `ftLib_800867E8`
pub unsafe extern "C" fn ftLib_800867E8(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLib_800867E8");
    let mut fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
    ext::Fighter_ResetInputData_80068854(gobj);
    (*fp).set_x221D_b4((1 as u16) as u16);
}

/// `ftLib_80086824`
pub unsafe extern "C" fn ftLib_80086824() {
    melee_core::ct!("ftLib_80086824");
    let mut __: [u8; 16] = [0; 16];
    let mut cur: *mut HSD_GObj = null_mut();
    {
        cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
        'l24: while !cur.is_null() {
            ftLib_800867E8(cur);
            cur = (*cur).next;
        }
    }
}

/// `ftLib_8008688C`
pub unsafe extern "C" fn ftLib_8008688C(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLib_8008688C");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    (*fp).set_x221D_b4((0 as u16) as u16);
}

/// `ftLib_800868A4`
pub unsafe extern "C" fn ftLib_800868A4() {
    melee_core::ct!("ftLib_800868A4");
    let mut cur: *mut HSD_GObj = null_mut();
    {
        cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
        'l25: while !cur.is_null() {
            ftLib_8008688C(cur);
            cur = (*cur).next;
        }
    }
}

/// `ftLib_800868D4`
pub unsafe extern "C" fn ftLib_800868D4(mut gobj: *mut HSD_GObj, mut arg1: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_800868D4");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if (((*fp).item_gobj != arg1) && ((*fp).x197C != arg1)) && ((*fp).x1980 != arg1) {
        return 1;
    }
    if (((((*fp).invisible() as i32) != 0) || (((*fp).x221E_b5() as i32) != 0)) || ((!((*fp).x221E_b3() != 0)) && ((*fp).item_gobj == arg1))) || ((!((*fp).x221E_b4() != 0)) && (((*fp).x197C == arg1) || ((*fp).x1980 == arg1))) {
        return 0;
    } else {
        return 1;
    }
}

/// `ftLib_80086960`
pub unsafe extern "C" fn ftLib_80086960(mut arg: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_80086960");
    if (!arg.is_null()) && (((*arg).classifier as i32) == 4) {
        return 1;
    } else {
        return 0;
    }
}

/// `ftLib_80086984`
pub unsafe extern "C" fn ftLib_80086984(mut gobj: *mut HSD_GObj) -> *mut CollData {
    melee_core::ct!("ftLib_80086984");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return Fighter_GetCollData(fp);
}

/// `vector_add`
pub unsafe extern "C" fn vector_add(mut dst: *mut Vec, mut src: *mut Vec, mut x: f32, mut y: f32, mut z: f32) {
    melee_core::ct!("vector_add");
    (*dst).x = (*src).x + x;
    (*dst).y = (*src).y + y;
    (*dst).z = (*src).z + z;
}

/// `ftLib_80086990`
pub unsafe extern "C" fn ftLib_80086990(mut gobj: *mut HSD_GObj, mut v: *mut Vec) {
    melee_core::ct!("ftLib_80086990");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut tmp: f32 = 0.5f32 * ((*fp).coll_data.ecb.top.y + (*fp).coll_data.ecb.bottom.y);
    vector_add(v, core::ptr::addr_of_mut!((*fp).cur_pos) as *mut Vec, 0 as f32, tmp, 0 as f32);
}

/// `ftLib_800869D4`
pub unsafe extern "C" fn ftLib_800869D4(mut gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftLib_800869D4");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return ext::ftCommon_GetModelScale(fp);
}

/// `ftLib_800869F8`
pub unsafe extern "C" fn ftLib_800869F8(mut gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftLib_800869F8");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).x34_scale.x * (*fp).co_attrs.model_scaling;
}

/// `ftLib_80086A0C`
pub unsafe extern "C" fn ftLib_80086A0C(mut gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftLib_80086A0C");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).x34_scale.y;
}

/// `ftLib_80086A18`
pub unsafe extern "C" fn ftLib_80086A18(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_80086A18");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut result: i32 = 0;
    's26: {
        match (*fp).motion_id {
            178 | 179 | 181 => {
                result = 1;
            }
            _ => {}
        }
    }
    return result;
}

/// `ftLib_80086A4C`
pub unsafe extern "C" fn ftLib_80086A4C(mut gobj: *mut HSD_GObj, mut val: f32) {
    melee_core::ct!("ftLib_80086A4C");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    (*fp).dmg.x1958 = val;
}

/// `ftLib_80086A58`
pub unsafe extern "C" fn ftLib_80086A58(mut gobj: *mut HSD_GObj, mut x: *mut S32Vec2) -> i32 {
    melee_core::ct!("ftLib_80086A58");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if (!((*fp).x2229_b3() != 0)) && (!((*fp).x2220_b7() != 0)) {
        *x = (*fp).x2188;
    }
    return 0;
}

/// `ftLib_80086A8C`
pub unsafe extern "C" fn ftLib_80086A8C(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_80086A8C");
    return 1;
}

/// `ftLib_UpdateLogicOffCamera`
pub unsafe extern "C" fn ftLib_UpdateLogicOffCamera(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLib_UpdateLogicOffCamera");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if (*fp).x221F_b3() != 0 {
        return;
    }
    if ((!((*fp).x2229_b3() != 0)) && (!((*fp).x2220_b7() != 0))) && (!ext::Camera_80030A50().is_null()) {
        (*fp).set_x221F_b0((((!(ext::Camera_LogicToScreen((*fp).x890_cameraBox, core::ptr::addr_of_mut!((*fp).x2188) as *mut S32Vec2) != 0)) as i32) as u8) as u8);
    } else {
        (*fp).set_x221F_b0((0 as u8) as u8);
    }
}

/// `ftLib_80086B64`
pub unsafe extern "C" fn ftLib_80086B64(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_80086B64");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).x221F_b0() as i32;
}

/// `ftLib_80086B74`
pub unsafe extern "C" fn ftLib_80086B74(mut gobj: *mut HSD_GObj) -> *mut CmSubject {
    melee_core::ct!("ftLib_80086B74");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).x890_cameraBox;
}

/// `ftLib_80086B80`
pub unsafe extern "C" fn ftLib_80086B80(mut gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftLib_80086B80");
    return (*ftLib_80086B74(gobj)).target_ext.v.z;
}

/// `ftLib_80086B90`
pub unsafe extern "C" fn ftLib_80086B90(mut gobj: *mut HSD_GObj, mut v: *mut Vec) {
    melee_core::ct!("ftLib_80086B90");
    let mut cam: *mut CmSubject = ftLib_80086B74(gobj);
    *v = (*cam).bone_pos;
}

/// `ftLib_80086BB4`
pub unsafe extern "C" fn ftLib_80086BB4(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_80086BB4");
    let mut cam: *mut CmSubject = ftLib_80086B74(gobj);
    return ext::Camera_80031154(core::ptr::addr_of_mut!((*cam).pos) as *mut Vec);
}

/// `ftLib_80086BE0`
pub unsafe extern "C" fn ftLib_80086BE0(mut gobj: *mut HSD_GObj) -> u8 {
    melee_core::ct!("ftLib_80086BE0");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).player_id;
}

/// `ftLib_80086BEC`
pub unsafe extern "C" fn ftLib_80086BEC(mut gobj: *mut HSD_GObj, mut v: *mut Vec) {
    melee_core::ct!("ftLib_80086BEC");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    *v = (*fp).pos_delta;
}

/// `ftLib_GetMotionId`
pub unsafe extern "C" fn ftLib_GetMotionId(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_GetMotionId");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).motion_id;
}

/// `helper`
pub unsafe extern "C" fn helper(mut gobj: *mut HSD_GObj, mut arg1: i32, mut arg2: i32, mut val: i32) {
    melee_core::ct!("helper");
    let mut fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
    if !(ext::Player_8003544C((*fp).player_id as i32, (*fp).is_sub_fighter() as i32) != 0) {
        return;
    }
    if (!((*fp).x221F_b3() != 0)) && (!((*fp).x2224_b2() != 0)) {
        ext::lb_80014574((*fp).x618_player_id, val, arg1, arg2);
    }
}

/// `ftLib_80086C18`
pub unsafe extern "C" fn ftLib_80086C18(mut gobj: *mut HSD_GObj, mut arg1: i32, mut arg2: i32) {
    melee_core::ct!("ftLib_80086C18");
    helper(gobj, arg1, arg2, 0);
}

/// `ftLib_80086C9C`
pub unsafe extern "C" fn ftLib_80086C9C(mut arg0: i32, mut arg1: i32) {
    melee_core::ct!("ftLib_80086C9C");
    let mut __: [u8; 16] = [0; 16];
    let mut cur: *mut HSD_GObj = null_mut();
    {
        cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
        'l27: while !cur.is_null() {
            ftLib_80086C18(cur, arg0, arg1);
            cur = (*cur).next;
        }
    }
}

/// `ftLib_80086D40`
pub unsafe extern "C" fn ftLib_80086D40(mut arg0: *mut HSD_GObj, mut arg1: i32, mut arg2: i32) {
    melee_core::ct!("ftLib_80086D40");
    helper(arg0, arg1, arg2, 1);
}

/// `ftLib_80086DC4`
pub unsafe extern "C" fn ftLib_80086DC4(mut arg0: i32, mut arg1: i32) {
    melee_core::ct!("ftLib_80086DC4");
    let mut __: [u8; 16] = [0; 16];
    let mut cur: *mut HSD_GObj = null_mut();
    {
        cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
        'l28: while !cur.is_null() {
            ftLib_80086D40(cur, arg0, arg1);
            cur = (*cur).next;
        }
    }
}

/// `ftLib_80086E68`
pub unsafe extern "C" fn ftLib_80086E68(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLib_80086E68");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if ext::Player_8003544C((*fp).player_id as i32, (*fp).is_sub_fighter() as i32) != 0 {
        ext::HSD_PadRumbleRemoveId((*fp).x618_player_id, 1);
    }
}

/// `ftLib_80086EB4`
pub unsafe extern "C" fn ftLib_80086EB4(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_80086EB4");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).team as i32;
}

/// `ftLib_80086EC0`
pub unsafe extern "C" fn ftLib_80086EC0(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_80086EC0");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).x221C_b6() as i32;
}

/// `ftLib_80086ED0`
pub unsafe extern "C" fn ftLib_80086ED0(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_80086ED0");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if (((((((*fp).invisible() as i32) != 0) || (((*fp).x221E_b2() as i32) != 0)) || (ext::Player_GetMoreFlagsBit4((*fp).player_id as i32) != 0)) || (((*fp).is_sandbag() as i32) != 0)) || (((*fp).x2229_b3() as i32) != 0)) || (((*fp).x2220_b7() as i32) != 0) {
        return 0;
    }
    return 1;
}

/// `ftLib_80086F4C`
pub unsafe extern "C" fn ftLib_80086F4C(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_80086F4C");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if ((!((*fp).x221F_b2() != 0)) && (!((*fp).x221F_b1() != 0))) && (((*fp).x209A as i32) == 1) {
        return 1;
    }
    return 0;
}

/// `ftLib_80086F80`
pub unsafe extern "C" fn ftLib_80086F80(mut gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftLib_80086F80");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if (*fp).invisible() != 0 {
        return (*(*(*fp).ft_data).x0).name_tag_height;
    }
    return (*fp).co_attrs.name_tag_height;
}

/// `ftLib_80086FA8`
pub unsafe extern "C" fn ftLib_80086FA8(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_80086FA8");
    's29: {
        match ftLib_GetMotionId(gobj) {
            120 | 121 | 122 | 123 => {
                return 1;
            }
            _ => {}
        }
    }
    return 0;
}

/// `ftLib_80086FD4`
pub unsafe extern "C" fn ftLib_80086FD4(mut gobj0: *mut HSD_GObj, mut gobj1: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_80086FD4");
    if (!gobj0.is_null()) && (!gobj1.is_null()) {
        if gobj0 == gobj1 {
            return 1;
        }
        {
            let mut fp0: *mut Fighter = HSD_GObjGetUserData(gobj0) as *mut Fighter;
            let mut fp1: *mut Fighter = HSD_GObjGetUserData(gobj1) as *mut Fighter;
            if ((*fp0).player_id as i32) == ((*fp1).player_id as i32) {
                return 1;
            }
        }
    }
    return 0;
}

/// `ftLib_8008701C`
pub unsafe extern "C" fn ftLib_8008701C(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_8008701C");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).x221F_b3() as i32;
}

/// `ftLib_8008702C`
pub unsafe extern "C" fn ftLib_8008702C(mut arg0: i32) {
    melee_core::ct!("ftLib_8008702C");
    ext::ftData_80085560(arg0, 1);
}

/// `ftLib_80087050`
pub unsafe extern "C" fn ftLib_80087050(mut arg0: i32) {
    melee_core::ct!("ftLib_80087050");
    ext::ftData_80085560(arg0, (-1_i32));
}

/// `ftLib_80087074`
pub unsafe extern "C" fn ftLib_80087074(mut gobj: *mut HSD_GObj, mut v: *mut Vec) -> i32 {
    melee_core::ct!("ftLib_80087074");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if ((*fp).dmg.x18B8 != 0.0) || ((*fp).dmg.x18BC != 0.0) {
        (*v).x = (*fp).dmg.x18B8;
        (*v).y = (*fp).dmg.x18BC;
        (*v).z = 0.0f32;
        return 1;
    }
    return 0;
}

/// `ftLib_800870BC`
pub unsafe extern "C" fn ftLib_800870BC(mut gobj: *mut HSD_GObj, mut val: *mut i32) -> i32 {
    melee_core::ct!("ftLib_800870BC");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if (*fp).x61A_controller_index != 0 {
        *val = (*ext::p_ftCommonData).x6D8[(*fp).x61A_controller_index as usize];
        return 1;
    }
    return 0;
}

/// `ftLib_800870F0`
pub unsafe extern "C" fn ftLib_800870F0(mut gobj: *mut HSD_GObj, mut x: i32) {
    melee_core::ct!("ftLib_800870F0");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    (*fp).dmg.x1830_percent = x as f32;
}

/// `ftLib_80087120`
pub unsafe extern "C" fn ftLib_80087120(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_80087120");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut result: i32 = (*fp).dmg.x1830_percent as i32;
    return result;
}

/// `ftLib_80087140`
pub unsafe extern "C" fn ftLib_80087140(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLib_80087140");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if !((*fp).x221F_b3() != 0) {
        ext::ftCo_800D4F24(gobj, 1);
        ext::ftCommon_8007ED2C(fp);
        ext::Fighter_UnkProcessDeath_80068354(gobj);
        ext::ftCommon_8007D92C(gobj);
    }
}

/// `ftLib_800871A8`
pub unsafe extern "C" fn ftLib_800871A8(mut gobj: *mut HSD_GObj, mut item_gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLib_800871A8");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if ext::itGetKind(item_gobj) == It_Kind_MetalB { (); } else { panic!("ftLib_800871A8: assertion failed"); }
    ext::ftCo_800C8348(gobj, ext::it_8026B560(item_gobj) as i32, ext::it_8026B574(item_gobj) as i32);
    ext::ftCo_800D105C(gobj);
    ext::ft_80081C88(gobj, (*fp).x34_scale.y);
    ext::ftCommon_8007EBAC(fp, 12 as u32, 0 as u32);
    ext::Camera_RequestQuake((QuakeKind_Small as i32) as u32, core::ptr::addr_of_mut!((*fp).cur_pos) as *mut Vec);
    ext::ft_PlaySFX(fp, 289, 127 as u8, 64 as u8);
}

/// `ftLib_80087284`
pub unsafe extern "C" fn ftLib_80087284(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_80087284");
    if ftLib_GetMotionId(gobj) >= ftCo_MS_LightThrowF4 {
        return 1;
    }
    return 0;
}

/// `ftLib_GetKind`
pub unsafe extern "C" fn ftLib_GetKind(mut gobj: *mut HSD_GObj) -> FighterKind {
    melee_core::ct!("ftLib_GetKind");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    {
        if ((*fp).kind >= ((Ft_Kind_Mex0 as i32) as u32)) && (ext::Geno_DefineTemplateSkeleton((*fp).kind as i32) != 0) {
            return ext::Geno_DefineBaseKind((*fp).kind as i32) as u32;
        }
    }
    return (*fp).kind;
}

/// `ftLib_800872B0`
pub unsafe extern "C" fn ftLib_800872B0(mut gobj: *mut HSD_GObj) -> *mut LbShadow {
    melee_core::ct!("ftLib_800872B0");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return core::ptr::addr_of_mut!((*fp).x20A4) as *mut LbShadow;
}

/// `ftLib_800872BC`
pub unsafe extern "C" fn ftLib_800872BC(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_800872BC");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (((!((*fp).x221E_b1() != 0)) && (!((*fp).x2226_b4() != 0))) && (!((*fp).is_sub_fighter() != 0))) as i32;
}

/// `ftLib_80087300`
pub unsafe extern "C" fn ftLib_80087300(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_80087300");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).dmg.x18c4_source_ply;
}

/// `ftLib_8008730C`
pub unsafe extern "C" fn ftLib_8008730C(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_8008730C");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).x221F_b5() as i32;
}

/// `ftLib_8008731C`
pub unsafe extern "C" fn ftLib_8008731C(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_8008731C");
    return ftLib_8008701C(gobj);
}

/// `ftLib_8008732C`
pub unsafe extern "C" fn ftLib_8008732C(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_8008732C");
    let mut msid: i32 = ftLib_GetMotionId(gobj);
    if (msid >= ftCo_MS_DeadDown) && (msid <= ftCo_MS_DeadUpFallHitCameraIce) {
        return 1;
    }
    return 0;
}

/// `ftLib_80087354`
pub unsafe extern "C" fn ftLib_80087354(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_80087354");
    let mut msid: i32 = ftLib_GetMotionId(gobj);
    if (msid >= ftCo_MS_DeadUp) && (msid <= ftCo_MS_DeadUpFallHitCameraIce) {
        return 1;
    }
    return 0;
}

/// `ftLib_8008737C`
pub unsafe extern "C" fn ftLib_8008737C(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_8008737C");
    let mut msid: i32 = ftLib_GetMotionId(gobj);
    if (msid >= ftCo_MS_DeadUpStar) && (msid <= ftCo_MS_DeadUpStarIce) {
        return 1;
    }
    return 0;
}

/// `ftLib_800873A4`
pub unsafe extern "C" fn ftLib_800873A4(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_800873A4");
    let mut msid: i32 = ftLib_GetMotionId(gobj);
    if (msid >= ftCo_MS_DeadUpFall) && (msid <= ftCo_MS_DeadUpFallHitCameraIce) {
        return 1;
    }
    return 0;
}

/// `ftLib_800873CC`
pub unsafe extern "C" fn ftLib_800873CC(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_800873CC");
    let mut msid: i32 = ftLib_GetMotionId(gobj);
    if (msid >= ftCo_MS_Rebirth) && (msid <= ftCo_MS_RebirthWait) {
        return 1;
    }
    return 0;
}

/// `ftLib_800873F4`
pub unsafe extern "C" fn ftLib_800873F4(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_800873F4");
    let mut msid: i32 = ftLib_GetMotionId(gobj);
    if (msid >= ftCo_MS_Entry) && (msid <= ftCo_MS_EntryEnd) {
        return 1;
    }
    return 0;
}

/// `ftLib_8008741C`
pub unsafe extern "C" fn ftLib_8008741C(mut i: u32) -> *mut HSD_GObj {
    melee_core::ct!("ftLib_8008741C");
    let mut cur: *mut HSD_GObj = null_mut();
    {
        cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
        'l30: while !cur.is_null() {
            let mut fp: *mut Fighter = HSD_GObjGetUserData(cur) as *mut Fighter;
            if ((*fp).x8_spawnNum as u32) == i {
                return cur;
            }
            cur = (*cur).next;
        }
    }
    return null_mut();
}

/// `ftLib_80087454`
pub unsafe extern "C" fn ftLib_80087454(mut gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftLib_80087454");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).dmg.x18A4_knockbackMagnitude;
}

/// `ftLib_80087460`
pub unsafe extern "C" fn ftLib_80087460(mut gobj: *mut HSD_GObj) -> u32 {
    melee_core::ct!("ftLib_80087460");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).x8_spawnNum as u32;
}

/// `ftLib_8008746C`
pub unsafe extern "C" fn ftLib_8008746C(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_8008746C");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if ext::ftCommon_80080144(fp) != 0 {
        return 130001;
    }
    {
        let mut id: i32 = (*(*(*fp).ft_data).x4C_sfx).x34;
        if (id >= 5000) && (id < 10000) {
            let mut ssm: i32 = ext::Mex_SsmForPortKind((*fp).kind as i32);
            if ssm >= 0 {
                return (id - 5000) + (ssm * 10000);
            }
        }
        return id;
    }
    return (*(*(*fp).ft_data).x4C_sfx).x34;
}

/// `ftLib_800874BC`
pub unsafe extern "C" fn ftLib_800874BC(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_800874BC");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).is_sub_fighter() as i32;
}

/// `ftLib_800874CC`
pub unsafe extern "C" fn ftLib_800874CC(mut gobj: *mut HSD_GObj, mut arg1: *mut c_void, mut arg2: i32) {
    melee_core::ct!("ftLib_800874CC");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    ext::pl_8003E4A4((*fp).player_id as i32, (*fp).is_sub_fighter() as i32, arg1, arg2);
}

/// `ftLib_80087508`
pub unsafe extern "C" fn ftLib_80087508(mut ft_kind: i8, mut arg1: u8) {
    melee_core::ct!("ftLib_80087508");
    ext::ftData_8008572C(ft_kind as u32);
    ext::efAsync_LoadSync(ext::ftData_UnkBytePerCharacter[ft_kind as usize] as i32);
    ext::ftData_80085820(ft_kind as u32, arg1 as i32);
    ext::ftData_80085A14(ft_kind as u32);
    ext::ftData_800857E0(ft_kind as u32);
}

/// `ftLib_80087574`
pub unsafe extern "C" fn ftLib_80087574(mut ft_kind: i8) {
    melee_core::ct!("ftLib_80087574");
    let mut i: u8 = 0;
    ext::ftData_8008572C(ft_kind as u32);
    ext::efAsync_LoadSync(ext::ftData_UnkBytePerCharacter[ft_kind as usize] as i32);
    {
        i = 0 as u8;
        'l31: while (i as i32) < (ext::CostumeListsForeachCharacter[ft_kind as usize].numCostumes as i32) {
            ext::ftData_80085820(ft_kind as u32, i as i32);
            i = i + 1;
        }
    }
    ext::ftData_80085A14(ft_kind as u32);
    ext::ftData_800857E0(ft_kind as u32);
}

/// `ftLib_80087610`
pub unsafe extern "C" fn ftLib_80087610(mut arg0: u8) {
    melee_core::ct!("ftLib_80087610");
    let mut i: u8 = 0;
    {
        i = 0 as u8;
        'l32: while (i as i32) <= (SELKIND_COUNT as i32) {
            'c32: {
                if (i as i32) >= (SELKIND_COUNT as i32) {
                    break 'c32;
                }
                {
                    ext::ftKb_SpecialN_800EED50(ext::Player_800325C8(i as u32, 0) as i32, arg0 as i32);
                    if ((i as i32) == (CKind_Zelda as i32)) || ((i as i32) == (CKind_Seak as i32)) {
                        ext::ftKb_SpecialN_800EED50(ext::Player_800325C8(i as u32, 1) as i32, arg0 as i32);
                    }
                }
            }
            i = i + 1;
        }
    }
}

/// `ftLib_800876B4`
pub unsafe extern "C" fn ftLib_800876B4(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLib_800876B4");
    let _ = ext::ftAnim_IsFramesRemaining(gobj);
}

/// `ftLib_800876D4`
pub unsafe extern "C" fn ftLib_800876D4(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_800876D4");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if (*fp).smash_attrs.state == (2 as u32) {
        return 1;
    } else {
        return 0;
    }
}

/// `ftLib_800876F4`
pub unsafe extern "C" fn ftLib_800876F4(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_800876F4");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).dmg.x18CC;
}

/// `ftLib_80087700`
pub unsafe extern "C" fn ftLib_80087700(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_80087700");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    return (*fp).dmg.x18D0;
}

/// `ftLib_8008770C`
pub unsafe extern "C" fn ftLib_8008770C(mut gobj: *mut HSD_GObj, mut dst: *mut c_void) {
    melee_core::ct!("ftLib_8008770C");
    if ftLib_GetKind(gobj) == ((Ft_Kind_GameWatch as i32) as u32) {
        ext::ftGw_Init_8014A7F4(gobj, dst as *mut ItemModStruct);
    } else {
        ext::ftKb_Init_800EEB00(gobj, dst as *mut *mut ArticleDynamicBones);
    }
}

/// `ftLib_80087744`
pub unsafe extern "C" fn ftLib_80087744(mut gobj: *mut HSD_GObj, mut dst: *mut c_void) {
    melee_core::ct!("ftLib_80087744");
    if ftLib_GetKind(gobj) == ((Ft_Kind_GameWatch as i32) as u32) {
        ext::ftGw_Init_8014A814(gobj, dst as *mut ItemModStruct);
    } else {
        ext::ftKb_Init_800EEB1C(gobj, dst as *mut i32);
    }
}

/// `ftLib_8008777C`
pub unsafe extern "C" fn ftLib_8008777C(mut gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftLib_8008777C");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if (*fp).ground_or_air == ((GA_Ground as i32) as u32) { (); } else { panic!("ftLib_8008777C: assertion failed"); }
    {
        let mut cd: *mut CollData = Fighter_GetCollData(fp);
        return ext::atan2f(-(*cd).floor.normal.x, (*cd).floor.normal.y);
    }
}

/// `ftLib_800877D4`
pub unsafe extern "C" fn ftLib_800877D4(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_800877D4");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    if (*fp).xF8_playerNudgeVel.x != (0 as f32) {
        return 1;
    } else {
        return 0;
    }
}

/// `HSD_GObjGetUserData`
pub unsafe extern "C" fn HSD_GObjGetUserData(mut gobj: *mut HSD_GObj) -> *mut c_void {
    melee_core::ct!("HSD_GObjGetUserData");
    return (*gobj).user_data;
}

/// `Fighter_GetCollData`
pub unsafe extern "C" fn Fighter_GetCollData(mut fp: *mut Fighter) -> *mut CollData {
    melee_core::ct!("Fighter_GetCollData");
    return core::ptr::addr_of_mut!((*fp).coll_data) as *mut CollData;
}
