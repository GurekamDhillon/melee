//! src/melee/ft/kinds/ftKirby/ftkirbyspeciallink.c: Kirby's Link / Young Link copy ability (bow).
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function). Locals the C leaves
//! uninitialised on a switch default read 0 (`-ftrivial-auto-var-init=zero`), kept here.
//! Smoke-tested by tests/tu_ftkirbyspeciallink.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (round 2): re-exported under this TU.
pub use crate::chars::r2::ftKirby::ftkirbyspeciallink::{
    ftKb_LkSpecialAirNEnd_Phys, ftKb_LkSpecialAirNLoop_Phys, ftKb_LkSpecialAirNStart_Phys,
    ftKb_LkSpecialNEnd_Phys, ftKb_LkSpecialNLoop_Phys, ftKb_LkSpecialNStart_Phys,
};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, off, out),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, off as _, out as _); } return; },
                }
            }
    pub static mut IT_802A83E0: Option<fn(f32, *mut HSD_GObj, *mut Vec3, Fighter_Part, i32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802A83E0(facing_dir: f32, gobj: *mut HSD_GObj, pos: *mut Vec3, part: Fighter_Part, loader_id: i32) -> *mut HSD_GObj {
                match unsafe { IT_802A83E0 } {
                    Some(f) => f(facing_dir, gobj, pos, part, loader_id),
                    None => { return unsafe { melee_items::tu::itlinkarrow::it_802A83E0(facing_dir as _, gobj as _, pos as _, part as _, loader_id as _) as _ }; },
                }
            }
    pub static mut IT_802AF1A4: Option<fn(f32, *mut HSD_GObj, *mut Vec3, Fighter_Part, i32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802AF1A4(facing_dir: f32, gobj: *mut HSD_GObj, pos: *mut Vec3, part: Fighter_Part, item_id: i32) -> *mut HSD_GObj {
                match unsafe { IT_802AF1A4 } {
                    Some(f) => f(facing_dir, gobj, pos, part, item_id),
                    None => { return unsafe { melee_items::tu::itlinkbow::it_802AF1A4(facing_dir as _, gobj as _, pos as _, part as _, item_id as _) as _ }; },
                }
            }
    pub static mut ITLINKARROW_802A850C: Option<fn(*mut HSD_GObj, *mut Vec3, *mut Vec3, f32, f32, f32) -> bool> = None;
            #[allow(non_snake_case)]
            pub fn itLinkArrow_802A850C(arrow: *mut HSD_GObj, pos1: *mut Vec3, pos2: *mut Vec3, angle: f32, charge: f32, max: f32) -> bool {
                match unsafe { ITLINKARROW_802A850C } {
                    Some(f) => f(arrow, pos1, pos2, angle, charge, max),
                    None => { return unsafe { melee_items::tu::itlinkarrow::itLinkArrow_802A850C(arrow as _, pos1 as _, pos2 as _, angle as _, charge as _, max as _) != 0 }; },
                }
            }
    pub static mut IT_802A8398: Option<fn(*mut HSD_GObj, *mut Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn it_802A8398(arrow: *mut HSD_GObj, b: *mut Vec3, a: *mut Vec3)  {
                match unsafe { IT_802A8398 } {
                    Some(f) => f(arrow, b, a),
                    None => { unsafe { let _ = melee_items::tu::itlinkarrow::it_802A8398(arrow as _, b as _, a as _); } return; },
                }
            }
    pub static mut IT_802A8A7C: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802A8A7C(arrow: *mut HSD_GObj)  {
                match unsafe { IT_802A8A7C } {
                    Some(f) => f(arrow),
                    None => { unsafe { let _ = melee_items::tu::itlinkarrow::it_802A8A7C(arrow as _); } return; },
                }
            }
    pub static mut IT_802AF304: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802AF304(bow: *mut HSD_GObj)  {
                match unsafe { IT_802AF304 } {
                    Some(f) => f(bow),
                    None => { unsafe { let _ = melee_items::tu::itlinkbow::it_802AF304(bow as _); } return; },
                }
            }
    pub static mut FTPICKUPITEM_80094818: Option<fn(*mut HSD_GObj, bool) > = None;
            #[allow(non_snake_case)]
            pub fn ftpickupitem_80094818(gobj: *mut HSD_GObj, flag: bool)  {
                match unsafe { FTPICKUPITEM_80094818 } {
                    Some(f) => f(gobj, flag),
                    None => { unsafe { let _ = melee_fighter::tu::ftpickupitem::ftpickupitem_80094818(gobj as _, flag as _); } return; },
                }
            }
    // `ftKb_Init_803B7548` (ftkirby.c data): the bow's three-point group.
    hook!(FTKB_INIT_803B7548, ftKb_Init_803B7548, () -> *const ftLk_SpecialN_Vec3Group);
}

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpp(gobj: G) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *const ftKb_DatAttrs {
    (*fp).dat_attrs as *const ftKb_DatAttrs
}

/// `ftKb_SpecialN_set_cbs` (ftKirby/inlines.h).
#[inline(always)]
unsafe fn set_cbs(gobj: G) {
    let fp = fpp(gobj);
    (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
    (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
}

/// Switch on the hat kind: Link, Young Link, or anything else.
#[inline(always)]
unsafe fn pick<T>(fp: *mut Fighter, lk: T, cl: T, other: T) -> T {
    let k = (*fp).u.kb.hat.kind as u32;
    if k == Ft_Kind_Link as u32 {
        lk
    } else if k == Ft_Kind_CLink as u32 {
        cl
    } else {
        other
    }
}

/// Link / Young Link motion id; any other hat keeps the Link id (the C default).
#[inline(always)]
unsafe fn ms(gobj: G, lk: u32, cl: u32) -> i32 {
    pick(fpp(gobj), lk, cl, lk) as i32
}

/// `ftKb_SpecialNLk_UnsetArrow` (static inline).
unsafe fn unset_arrow(gobj: G) {
    if !gobj.is_null() {
        let fp = fpp(gobj);
        if !fp.is_null() && !(*fp).u.kb.x74.is_null() {
            (*fp).u.kb.x74 = core::ptr::null_mut();
        }
    }
}

/// `ftKb_SpecialNLk_UnsetBow` (static inline).
unsafe fn unset_bow(gobj: G) {
    if !gobj.is_null() {
        let fp = fpp(gobj);
        if !fp.is_null() && !(*fp).u.kb.x78.is_null() {
            (*fp).u.kb.x78 = core::ptr::null_mut();
        }
    }
}

const LK_STATES: [u32; 6] = [
    ftKb_MS_LkSpecialNStart as u32,
    ftKb_MS_LkSpecialNLoop as u32,
    ftKb_MS_LkSpecialNEnd as u32,
    ftKb_MS_LkSpecialAirNStart as u32,
    ftKb_MS_LkSpecialAirNLoop as u32,
    ftKb_MS_LkSpecialAirNEnd as u32,
];
const CL_STATES: [u32; 6] = [
    ftKb_MS_ClSpecialNStart as u32,
    ftKb_MS_ClSpecialNLoop as u32,
    ftKb_MS_ClSpecialNEnd as u32,
    ftKb_MS_ClSpecialAirNStart as u32,
    ftKb_MS_ClSpecialAirNLoop as u32,
    ftKb_MS_ClSpecialAirNEnd as u32,
];

pub fn ftKb_SpecialNLk800FB394(gobj: G) -> ftLk_SpecialNIndex {
    unsafe {
        let mut idx = ftLk_SpecialNIndex_None;
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() && !(*fp).u.kb.x78.is_null() {
                let k = (*fp).u.kb.hat.kind as u32;
                let m = (*fp).motion_id as u32;
                if k == Ft_Kind_Link as u32 {
                    if LK_STATES.contains(&m) {
                        idx = m.wrapping_sub(ftKb_MS_LkSpecialNStart as u32);
                    }
                } else if k == Ft_Kind_CLink as u32 {
                    if CL_STATES.contains(&m) {
                        idx = m.wrapping_sub(ftKb_MS_ClSpecialNStart as u32);
                    }
                }
            }
        }
        idx
    }
}

pub fn ftKb_SpecialNLk800FB418(gobj: G) {
    unsafe { unset_arrow(gobj) }
}

pub fn ftKb_SpecialNLk800FB444(gobj: G) {
    unsafe { unset_bow(gobj) }
}

pub fn ftKb_SpecialNLk800FB470(gobj: G) -> bool {
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() {
                let k = (*fp).u.kb.hat.kind as u32;
                let m = (*fp).motion_id as u32;
                let in_set = if k == Ft_Kind_Link as u32 {
                    LK_STATES.contains(&m)
                } else if k == Ft_Kind_CLink as u32 {
                    CL_STATES.contains(&m)
                } else {
                    false
                };
                if in_set && (*fp).x2070.anon_0.x2071_b6() == 0 {
                    return false;
                }
            }
        }
        true
    }
}

pub fn ftKb_SpecialNLk800FB500(gobj: G) -> bool {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).cmd_vars[0] == 1 && (*fp).u.kb.x74.is_null() {
            let mut pos: Vec3 = core::mem::zeroed();
            (*fp).cmd_vars[0] = 0;
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_RHandN as usize)).joint, core::ptr::null(), &mut pos);
            let loader_id = pick(fp, (*da).specialn_lk_bow_item_loader_id, (*da).specialn_cl_bow_item_loader_id, 0) as i32;
            (*fp).u.kb.x74 = ext::it_802A83E0((*fp).facing_dir, gobj, &mut pos, FtPart_RHandN, loader_id);
            if !(*fp).u.kb.x74.is_null() {
                set_cbs(gobj);
            } else {
                return true;
            }
        }
        false
    }
}

pub fn ftKb_SpecialNLk800FB5F4(gobj: G) -> bool {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).u.kb.x78.is_null() {
            let mut pos: Vec3 = core::mem::zeroed();
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_R3rdNa as usize)).joint, core::ptr::null(), &mut pos);
            let item_id = pick(fp, (*da).specialn_lk_bow_item_id, (*da).specialn_cl_bow_item_id, 0) as i32;
            (*fp).u.kb.x78 = ext::it_802AF1A4((*fp).facing_dir, gobj, &mut pos, FtPart_R3rdNa, item_id);
            if !(*fp).u.kb.x78.is_null() {
                set_cbs(gobj);
            } else {
                return true;
            }
        } else {
            return true;
        }
        false
    }
}

pub fn ftKb_SpecialNLk800FB6DC(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).cmd_vars[1] == 1 && !(*fp).u.kb.x74.is_null() {
            let mut pos1: Vec3 = core::mem::zeroed();
            let mut pos2: Vec3 = core::mem::zeroed();
            (*fp).cmd_vars[1] = 0;
            ext::lb_8000B1CC((*(*fp).parts.add(44)).joint, core::ptr::null(), &mut pos1);
            ext::lb_8000B1CC((*(*fp).parts.add(39)).joint, core::ptr::null(), &mut pos2);
            pos2.z = 0.0;
            pos1.z = 0.0;
            let saved_item = (*fp).item_gobj;
            let max = pick(fp, (*da).specialn_lk_max_charge, (*da).specialn_cl_max_charge, 0.0);
            ext::itLinkArrow_802A850C((*fp).u.kb.x74, &mut pos1, &mut pos2, 0.08726646, (*fp).mv.lk.specialn.x0.y, max);
            unset_arrow(gobj);
            (*fp).item_gobj = saved_item;
            ext::ftpickupitem_80094818(gobj, false);
        }
    }
}

/// Callback entry for `ftKb_SpecialNLk800FB800`.
pub unsafe extern "C" fn cb_ftKb_SpecialNLk800FB800(g: G) {
    ftKb_SpecialNLk800FB800(g);
}
/// Callback entry for `ftKb_SpecialNLk800FB840`.
pub unsafe extern "C" fn cb_ftKb_SpecialNLk800FB840(g: G) {
    ftKb_SpecialNLk800FB840(g);
}

pub fn ftKb_SpecialNLk800FB800(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.kb.x74.is_null() {
            ext::it_802A8A7C((*fp).u.kb.x74);
            (*fp).u.kb.x74 = core::ptr::null_mut();
        }
    }
}

pub fn ftKb_SpecialNLk800FB840(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.kb.x78.is_null() {
            ext::it_802AF304((*fp).u.kb.x78);
            (*fp).u.kb.x78 = core::ptr::null_mut();
        }
    }
}

pub fn ftKb_SpecialNLk800FB880(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).mv.lk.specialn.x0.y = 0.0;
        (*fp).mv.lk.specialn.x0.x = 0.0;
        (*fp).mv.lk.specialn.unk_timer = 0;
        st::Fighter_ClearCmdVars(&mut *fp);
        st::ftCommon_8007D7FC(&mut *fp);
        (*fp).self_vel.y = 0.0;
        let msid = ms(gobj, ftKb_MS_LkSpecialNStart as u32, ftKb_MS_ClSpecialNStart as u32);
        st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        set_cbs(gobj);
        let anim_rate = pick(fp, (*da).specialn_lk_arrow_charge_speed, (*da).specialn_cl_arrow_charge_speed, 0.0);
        st::ftAnim_SetAnimRate(gobj, anim_rate);
        st::ftAnim_8006EBA4(gobj);
        if ftKb_SpecialNLk800FB5F4(gobj) {
            unset_arrow(gobj);
            unset_bow(gobj);
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftKb_SpecialNLk800FBA00(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).mv.lk.specialn.x0.y = 0.0;
        (*fp).mv.lk.specialn.x0.x = 0.0;
        (*fp).mv.lk.specialn.unk_timer = 0;
        st::Fighter_ClearCmdVars(&mut *fp);
        let msid = ms(gobj, ftKb_MS_LkSpecialAirNStart as u32, ftKb_MS_ClSpecialAirNStart as u32);
        st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        set_cbs(gobj);
        let anim_rate = pick(fp, (*da).specialn_lk_arrow_charge_speed, (*da).specialn_cl_arrow_charge_speed, 0.0);
        st::ftAnim_SetAnimRate(gobj, anim_rate);
        st::ftAnim_8006EBA4(gobj);
        if ftKb_SpecialNLk800FB5F4(gobj) {
            unset_arrow(gobj);
            unset_bow(gobj);
            let anim_rate = pick(fp, (*da).specialn_lk_freefall_toggle, (*da).specialn_cl_freefall_toggle, anim_rate);
            if anim_rate == 0.0 {
                st::ftCo_Fall_Enter(gobj);
                return;
            }
            st::ftCo_80096900(gobj, 1, 0, 1, 1.0, anim_rate);
        }
    }
}

/// The aim-vector refresh shared by the Start/Loop Anim callbacks: reads the (zero-filled in C
/// source) `ftKb_Init_803B7548` group.
unsafe fn aim_update(fp: *mut Fighter) {
    let vg = &*ext::ftKb_Init_803B7548();
    let a = vg.a;
    let mut b = vg.b;
    let c = vg.c;
    let mut a = a;
    (*fp).mv.lk.specialn.x8.y = b.x - c.x;
    (*fp).mv.lk.specialn.x8.z = b.y - c.y;
    (*fp).mv.lk.specialn.x14 = 0.0;
    (*fp).mv.lk.specialn.x8.x = melee_runtime::lbtrigf::atan2f(b.y - a.y, b.x - a.x);
    if !(*fp).u.kb.x74.is_null() {
        ext::it_802A8398((*fp).u.kb.x74, &mut b, &mut a);
    }
}

pub fn ftKb_LkSpecialNStart_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).cmd_vars[2] != 0 && (*fp).mv.lk.specialn.x0.x == 0.0 {
            (*fp).mv.lk.specialn.x0.x = 1.0;
            (*fp).cmd_vars[2] = 0;
        }
        let anim_rate = pick(fp, (*da).specialn_lk_arrow_charge_speed, (*da).specialn_cl_arrow_charge_speed, 0.0);
        st::ftAnim_SetAnimRate(gobj, anim_rate);
        if !ftKb_SpecialNLk800FB500(gobj) {
            let fp2 = fpp(gobj);
            aim_update(fp2);
            if st::ftAnim_IsFramesRemaining(gobj) == 0 {
                let msid = ms(gobj, ftKb_MS_LkSpecialNLoop as u32, ftKb_MS_ClSpecialNLoop as u32);
                st::Fighter_ChangeMotionState(gobj, msid, 0x40010, 0.0, 1.0, 0.0, core::ptr::null_mut());
                set_cbs(gobj);
                st::ftAnim_8006EBA4(gobj);
            }
        } else {
            unset_arrow(gobj);
            unset_bow(gobj);
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftKb_LkSpecialNLoop_Anim(gobj: G) {
    unsafe { aim_update(fpp(gobj)) }
}

pub fn ftKb_LkSpecialNEnd_Anim(gobj: G) {
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            unset_bow(gobj);
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftKb_LkSpecialAirNStart_Anim(gobj: G) {
    unsafe {
        let mut fp = fpp(gobj);
        let da = da(fp);
        if (*fp).cmd_vars[2] != 0 && (*fp).mv.lk.specialn.x0.x == 0.0 {
            (*fp).mv.lk.specialn.x0.x = 1.0;
            (*fp).cmd_vars[2] = 0;
        }
        if !ftKb_SpecialNLk800FB500(gobj) {
            fp = fpp(gobj);
            aim_update(fp);
            if st::ftAnim_IsFramesRemaining(gobj) == 0 {
                let msid = ms(gobj, ftKb_MS_LkSpecialAirNLoop as u32, ftKb_MS_ClSpecialAirNLoop as u32);
                st::Fighter_ChangeMotionState(gobj, msid, 0x40010, 0.0, 1.0, 0.0, core::ptr::null_mut());
                set_cbs(gobj);
                st::ftAnim_8006EBA4(gobj);
            }
        } else {
            unset_arrow(gobj);
            unset_bow(gobj);
            let anim_rate = pick(fp, (*da).specialn_lk_freefall_toggle, (*da).specialn_cl_freefall_toggle, 0.0);
            if anim_rate == 0.0 {
                st::ftCo_Fall_Enter(gobj);
                return;
            }
            st::ftCo_80096900(gobj, 1, 0, 1, 1.0, anim_rate);
        }
    }
}

pub fn ftKb_LkSpecialAirNLoop_Anim(gobj: G) {
    unsafe { aim_update(fpp(gobj)) }
}

pub fn ftKb_LkSpecialAirNEnd_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            unset_arrow(gobj);
            unset_bow(gobj);
            let freefall = pick(fp, (*da).specialn_lk_freefall_toggle, (*da).specialn_cl_freefall_toggle, 0.0);
            if freefall == 0.0 {
                st::ftCo_Fall_Enter(gobj);
            } else {
                st::ftCo_80096900(gobj, 1, 0, 1, 1.0, freefall);
            }
        }
    }
}

/// Charge-while-held body shared by the two Start IASA callbacks; `end_lk`/`end_cl` are the end states.
unsafe fn start_iasa(gobj: G, end_lk: u32, end_cl: u32) {
    let fp = fpp(gobj);
    let da = da(fp);
    if (*fp).mv.lk.specialn.x0.x != 0.0 {
        (*fp).mv.lk.specialn.x0.y += 1.0;
        let max = pick(fp, (*da).specialn_lk_max_charge, (*da).specialn_cl_max_charge, 0.0);
        if (*fp).mv.lk.specialn.x0.y > max {
            (*fp).mv.lk.specialn.x0.y = max;
        }
        if (*fp).input.held_buttons[0] & (HSD_PAD_B as u32) == 0 {
            let msid = ms(gobj, end_lk, end_cl);
            st::Fighter_ChangeMotionState(gobj, msid, 0x40010, 0.0, 1.0, 0.0, core::ptr::null_mut());
            set_cbs(gobj);
        }
    }
}

/// Full-charge body shared by the two Loop IASA callbacks.
unsafe fn loop_iasa(gobj: G, end_lk: u32, end_cl: u32) {
    let fp = fpp(gobj);
    let da = da(fp);
    let k = (*fp).u.kb.hat.kind as u32;
    if k == Ft_Kind_Link as u32 {
        (*fp).mv.lk.specialn.x0.y = (*da).specialn_lk_max_charge;
    } else if k == Ft_Kind_CLink as u32 {
        (*fp).mv.lk.specialn.x0.y = (*da).specialn_cl_max_charge;
    }
    if (*fp).input.held_buttons[0] & (HSD_PAD_B as u32) == 0 {
        let msid = ms(gobj, end_lk, end_cl);
        st::Fighter_ChangeMotionState(gobj, msid, 0x40010, 0.0, 1.0, 0.0, core::ptr::null_mut());
        set_cbs(gobj);
    }
}

pub fn ftKb_LkSpecialNStart_IASA(gobj: G) {
    unsafe { start_iasa(gobj, ftKb_MS_LkSpecialNEnd as u32, ftKb_MS_ClSpecialNEnd as u32) }
}
pub fn ftKb_LkSpecialNLoop_IASA(gobj: G) {
    unsafe { loop_iasa(gobj, ftKb_MS_LkSpecialNEnd as u32, ftKb_MS_ClSpecialNEnd as u32) }
}
pub fn ftKb_LkSpecialNEnd_IASA(_gobj: G) {}
pub fn ftKb_LkSpecialAirNStart_IASA(gobj: G) {
    unsafe { start_iasa(gobj, ftKb_MS_LkSpecialAirNEnd as u32, ftKb_MS_ClSpecialAirNEnd as u32) }
}
pub fn ftKb_LkSpecialAirNLoop_IASA(gobj: G) {
    unsafe { loop_iasa(gobj, ftKb_MS_LkSpecialAirNEnd as u32, ftKb_MS_ClSpecialAirNEnd as u32) }
}
pub fn ftKb_LkSpecialAirNEnd_IASA(_gobj: G) {}

const COLL_FLAGS: u32 = 0x0C4C5090;

/// Ground -> air Coll body; `lk`/`cl` are the air states.
unsafe fn coll_to_air(gobj: G, pre: Option<fn(G)>, lk: u32, cl: u32) {
    let fp = fpp(gobj);
    if let Some(p) = pre {
        p(gobj);
    }
    if st::ft_80082708(gobj) == GA_Ground as i32 {
        st::ftCommon_8007D5D4(&mut *fp);
        let anim_frame = (*fp).cur_anim_frame;
        let msid = ms(gobj, lk, cl);
        st::Fighter_ChangeMotionState(gobj, msid, COLL_FLAGS, anim_frame, 1.0, 0.0, core::ptr::null_mut());
        set_cbs(gobj);
        st::ftAnim_8006EBA4(gobj);
    }
}

/// Air -> ground Coll body.
unsafe fn coll_to_ground(gobj: G, pre: Option<fn(G)>, lk: u32, cl: u32) {
    let fp = fpp(gobj);
    if let Some(p) = pre {
        p(gobj);
    }
    if st::ft_80081D0C(gobj) == GA_Air as i32 {
        st::ftCommon_8007D7FC(&mut *fp);
        let anim_frame = (*fp).cur_anim_frame;
        let msid = ms(gobj, lk, cl);
        st::Fighter_ChangeMotionState(gobj, msid, COLL_FLAGS, anim_frame, 1.0, 0.0, core::ptr::null_mut());
        set_cbs(gobj);
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftKb_LkSpecialNStart_Coll(gobj: G) {
    unsafe { coll_to_air(gobj, None, ftKb_MS_LkSpecialAirNStart as u32, ftKb_MS_ClSpecialAirNStart as u32) }
}
pub fn ftKb_LkSpecialNLoop_Coll(gobj: G) {
    unsafe { coll_to_air(gobj, None, ftKb_MS_LkSpecialAirNLoop as u32, ftKb_MS_ClSpecialAirNLoop as u32) }
}
pub fn ftKb_LkSpecialNEnd_Coll(gobj: G) {
    unsafe {
        coll_to_air(
            gobj,
            Some(ftKb_SpecialNLk800FB6DC),
            ftKb_MS_LkSpecialAirNEnd as u32,
            ftKb_MS_ClSpecialAirNEnd as u32,
        )
    }
}
pub fn ftKb_LkSpecialAirNStart_Coll(gobj: G) {
    unsafe { coll_to_ground(gobj, None, ftKb_MS_LkSpecialNStart as u32, ftKb_MS_ClSpecialNStart as u32) }
}
pub fn ftKb_LkSpecialAirNLoop_Coll(gobj: G) {
    unsafe { coll_to_ground(gobj, None, ftKb_MS_LkSpecialNLoop as u32, ftKb_MS_ClSpecialNLoop as u32) }
}
pub fn ftKb_LkSpecialAirNEnd_Coll(gobj: G) {
    unsafe {
        coll_to_ground(
            gobj,
            Some(ftKb_SpecialNLk800FB6DC),
            ftKb_MS_LkSpecialNEnd as u32,
            ftKb_MS_ClSpecialNEnd as u32,
        )
    }
}
