//! src/melee/ft/kinds/ftLink/ftlinkspecialn.c: Link's Hero's Bow (neutral B).
//! No fused sites (tools/rust-port/fma_sites.py): the float arithmetic is subtractions, a
//! counter increment and the `5 * 0.01745329252f` constant (`MTXDegToRad(5)`). `atan2f` is group
//! 1's MSL port. Smoke-tested by tests/tu_ftlinkspecialn.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use crate::tu::ftlink::{ftLk_Init_BoomerangExists, Fighter_SetDamageCallback_ftLk_800EAF58};
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (round 2): re-exported under this TU.


/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    pub use crate::tu::ftlinkspeciallw::ext::{lb_8000B1CC, LB_8000B1CC};
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    // it/kinds/itlinkarrow.c, itlinkbow.c
    pub static mut IT_802A8A7C: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802A8A7C(arrow: *mut Item_GObj)  {
                match unsafe { IT_802A8A7C } {
                    Some(f) => f(arrow),
                    None => { unsafe { melee_items::tu::itlinkarrow::it_802A8A7C(arrow as _); } return; },
                }
            }
    pub static mut IT_802AF304: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802AF304(bow: *mut Item_GObj)  {
                match unsafe { IT_802AF304 } {
                    Some(f) => f(bow),
                    None => { unsafe { melee_items::tu::itlinkbow::it_802AF304(bow as _); } return; },
                }
            }
    crate::port_hook!(IT_802AF1A4, it_802AF1A4, crate::adapt::it_802AF1A4, (facing_dir: f32, owner: *mut HSD_GObj, pos: &Vec3, part: i32, x10: i32) -> *mut Item_GObj);
    crate::port_hook!(IT_802A83E0, it_802A83E0, crate::adapt::it_802A83E0, (facing_dir: f32, owner: *mut HSD_GObj, pos: &Vec3, part: i32, xc: i32) -> *mut Item_GObj);
    crate::port_hook!(IT_802A8398, it_802A8398, crate::adapt::it_802A8398, (arrow: *mut Item_GObj, rpos: &Vec3, lpos: &Vec3));
    crate::port_hook!(ITLINKARROW_802A850C, itLinkArrow_802A850C, crate::adapt::itLinkArrow_802A850C, (arrow: *mut Item_GObj, rpos: &Vec3, lpos: &Vec3, angle: f32, charge: f32, max_charge: f32));
}

const mf: u32 = (Ft_MF_SkipModel | Ft_MF_SkipItemVis) as u32;

const coll_mf: u32 = (Ft_MF_SkipModel
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

/// `MTXDegToRad(5)` (`(a) * 0.01745329252f`).
const DEG5_RAD: f32 = 5.0_f32 * 0.01745329252_f32;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline(always)]
unsafe fn da<'a>(fp: *mut Fighter) -> &'a ftLk_DatAttrs {
    &*((*fp).dat_attrs as *const ftLk_DatAttrs)
}

#[inline(always)]
unsafe fn bone_joint(fp: *mut Fighter, part: Fighter_Part) -> *mut core::ffi::c_void {
    let idx = fh::ftparts_get_bone_index(&*fp, part as i32);
    (*(*fp).parts.offset(idx as isize)).joint as *mut core::ffi::c_void
}

#[inline(always)]
unsafe fn bone_index(fp: *mut Fighter, part: Fighter_Part) -> i32 {
    fh::ftparts_get_bone_index(&*fp, part as i32)
}

/// `Fighter_SetDamageCallback(gobj, ftLk_800EAF58)` (ft/inlines.h): take_dmg_cb and death2_cb.
#[inline(always)]
fn set_damage_cb(gobj: *mut HSD_GObj) {
    Fighter_SetDamageCallback_ftLk_800EAF58(gobj);
}

pub fn ftLk_SpecialN_GetIndex(gobj: *mut HSD_GObj) -> ftLk_SpecialNIndex {
    melee_core::ct!("ftLk_SpecialN_GetIndex");
    let mut result = ftLk_SpecialNIndex_None;
    if !gobj.is_null() {
        unsafe {
            let fp = fpp(gobj);
            if !fp.is_null() && !(*fp).u.lk.x14.is_null() {
                let msid = (*fp).motion_id as u32;
                if msid == ftLk_MS_SpecialNStart as u32
                    || msid == ftLk_MS_SpecialNLoop as u32
                    || msid == ftLk_MS_SpecialNEnd as u32
                    || msid == ftLk_MS_SpecialAirNStart as u32
                    || msid == ftLk_MS_SpecialAirNLoop as u32
                    || msid == ftLk_MS_SpecialAirNEnd as u32
                {
                    result = msid - ftLk_MS_SpecialNStart as u32;
                }
            }
        }
    }
    result
}

pub fn ftLk_SpecialN_UnsetArrow(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialN_UnsetArrow");
    if !gobj.is_null() {
        unsafe {
            let fp = fpp(gobj);
            if !fp.is_null() && !(*fp).u.lk.arrow_gobj.is_null() {
                (*fp).u.lk.arrow_gobj = core::ptr::null_mut();
                if (*fp).u.lk.boomerang_gobj.is_null() && (*fp).u.lk.x14.is_null() {
                    ftLk_Init_BoomerangExists(gobj);
                }
            }
        }
    }
}

pub fn ftLk_SpecialN_UnsetFv14(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialN_UnsetFv14");
    if !gobj.is_null() {
        unsafe {
            let fp = fpp(gobj);
            if !fp.is_null() && !(*fp).u.lk.x14.is_null() {
                (*fp).u.lk.x14 = core::ptr::null_mut();
                if (*fp).u.lk.boomerang_gobj.is_null() && (*fp).u.lk.arrow_gobj.is_null() {
                    ftLk_Init_BoomerangExists(gobj);
                }
            }
        }
    }
}

/// `ftLk_SpecialN_IsActiveAnd2071b6(gobj)`: the fighter-only body is `chars::ftLink`'s.
pub fn ftLk_SpecialN_IsActiveAnd2071b6(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftLk_SpecialN_IsActiveAnd2071b6");
    unsafe { crate::chars::ftLink::ftlinkspecialn::ftLk_SpecialN_IsActiveAnd2071b6(&*fpp(gobj)) }
}

pub fn ftLk_SpecialN_ProcessFv10(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialN_ProcessFv10");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.lk.arrow_gobj.is_null() {
            ext::it_802A8A7C((*fp).u.lk.arrow_gobj);
            (*fp).u.lk.arrow_gobj = core::ptr::null_mut();
            ftLk_Init_BoomerangExists(gobj);
        }
    }
}

pub fn ftLk_SpecialN_ProcessFv14(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialN_ProcessFv14");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.lk.x14.is_null() {
            ext::it_802AF304((*fp).u.lk.x14);
            (*fp).u.lk.x14 = core::ptr::null_mut();
            ftLk_Init_BoomerangExists(gobj);
        }
    }
}

/// `isDrawback` (static inline): create the bow; true when it could not be created or exists.
fn isDrawback(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        if (*fp).u.lk.x14.is_null() {
            let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ext::lb_8000B1CC(bone_joint(fp, FtPart_RThumbNb), None, &mut pos);
            let fv_x14 = ext::it_802AF1A4((*fp).facing_dir, gobj, &pos, bone_index(fp, FtPart_RThumbNb), a.x10);
            (*fp).u.lk.x14 = fv_x14;
            if !fv_x14.is_null() {
                set_damage_cb(gobj);
            } else {
                return true;
            }
        } else {
            return true;
        }
        false
    }
}

/// `isDrawn` (static inline): create the arrow once the script asks for it.
fn isDrawn(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        if (*fp).cmd_vars[0] == 1 && (*fp).u.lk.arrow_gobj.is_null() {
            let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            (*fp).cmd_vars[0] = 0;
            ext::lb_8000B1CC(bone_joint(fp, FtPart_LThumbNb), None, &mut pos);
            let fv_x10 = ext::it_802A83E0((*fp).facing_dir, gobj, &pos, bone_index(fp, FtPart_LThumbNb), a.xC);
            (*fp).u.lk.arrow_gobj = fv_x10;
            if !fv_x10.is_null() {
                set_damage_cb(gobj);
            } else {
                return true;
            }
        }
        false
    }
}

/// `animate` / `animate_nopad` (static inline, same body): track the aim vector of the bow.
fn animate(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let mut lpos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut rpos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut root = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::lb_8000B1CC(bone_joint(fp, FtPart_LThumbNb), None, &mut lpos);
        ext::lb_8000B1CC(bone_joint(fp, FtPart_RThumbNb), None, &mut rpos);
        ext::lb_8000B1CC((*(*fp).parts).joint as *mut _, None, &mut root);
        (*fp).mv.lk.specialn.x8.y = rpos.x - root.x;
        (*fp).mv.lk.specialn.x8.z = rpos.y - root.y;
        (*fp).mv.lk.specialn.x14 = 0.0;
        (*fp).mv.lk.specialn.x8.x = melee_runtime::lbtrigf::atan2f(rpos.y - lpos.y, rpos.x - lpos.x);
        if !(*fp).u.lk.arrow_gobj.is_null() {
            ext::it_802A8398((*fp).u.lk.arrow_gobj, &rpos, &lpos);
        }
    }
}

pub fn ftLk_SpecialN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialN_Enter");
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        (*fp).mv.lk.specialn.x0.y = 0.0;
        (*fp).mv.lk.specialn.x0.x = 0.0;
        (*fp).mv.lk.specialn.unk_timer = 0;
        st::Fighter_ClearCmdVars(&mut *fp);
        st::ftCommon_8007D7FC(&mut *fp);
        (*fp).self_vel.y = 0.0;
        st::Fighter_ChangeMotionState(gobj, ftLk_MS_SpecialNStart as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        set_damage_cb(gobj);
        st::ftAnim_SetAnimRate(gobj, a.specialn_anim_rate);
        st::ftAnim_8006EBA4(gobj);
        if isDrawback(gobj) {
            ftLk_SpecialN_UnsetArrow(gobj);
            ftLk_SpecialN_UnsetFv14(gobj);
            st::ft_8008A2BC(gobj);
        }
    }
}

/// The tail of the aerial draw-back failures: Fall, or FallSpecial with `x8` landing lag.
unsafe fn air_abort(gobj: *mut HSD_GObj, a: &ftLk_DatAttrs) {
    if a.x8 == 0.0 {
        st::ftCo_Fall_Enter(gobj);
    } else {
        st::ftCo_80096900(gobj, 1, 0, 1, 1.0, a.x8);
    }
}

pub fn ftLk_SpecialAirN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialAirN_Enter");
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        (*fp).mv.lk.specialn.x0.y = 0.0;
        (*fp).mv.lk.specialn.x0.x = 0.0;
        (*fp).mv.lk.specialn.unk_timer = 0;
        st::Fighter_ClearCmdVars(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, ftLk_MS_SpecialAirNStart as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        set_damage_cb(gobj);
        st::ftAnim_SetAnimRate(gobj, a.specialn_anim_rate);
        st::ftAnim_8006EBA4(gobj);
        if isDrawback(gobj) {
            ftLk_SpecialN_UnsetArrow(gobj);
            ftLk_SpecialN_UnsetFv14(gobj);
            air_abort(gobj, a);
        }
    }
}

pub fn ftLk_SpecialNStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialNStart_Anim");
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        if (*fp).cmd_vars[2] != 0 && (*fp).mv.lk.specialn.x0.x == 0.0 {
            (*fp).mv.lk.specialn.x0.x = 1.0;
            (*fp).cmd_vars[2] = 0;
        }
        st::ftAnim_SetAnimRate(gobj, a.specialn_anim_rate);
        if !isDrawn(gobj) {
            animate(gobj);
            if st::ftAnim_IsFramesRemaining(gobj) == 0 {
                st::Fighter_ChangeMotionState(gobj, ftLk_MS_SpecialNLoop as i32, mf, 0.0, 1.0, 0.0, core::ptr::null_mut());
                set_damage_cb(gobj);
                st::ftAnim_8006EBA4(gobj);
            }
        } else {
            ftLk_SpecialN_UnsetArrow(gobj);
            ftLk_SpecialN_UnsetFv14(gobj);
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftLk_SpecialNLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialNLoop_Anim");
    animate(gobj);
}

/// `setupParts` (static inline).
fn setupParts(gobj: *mut HSD_GObj, arg1: i32) {
    st::ftParts_80074A4C(gobj, 2, arg1);
    st::ftParts_80074B0C(gobj, 2, arg1);
}

/// `updateParts` (static inline): once the script flags it, show/hide the held-item part.
fn updateParts(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x221E_b3() == 1 && (*fp).mv.lk.specialn.unk_timer == 0 {
            (*fp).mv.lk.specialn.unk_timer += 1;
            if !(*fp).item_gobj.is_null() {
                setupParts(gobj, 1);
            } else {
                setupParts(gobj, 0);
            }
        }
    }
}

pub fn ftLk_SpecialNEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialNEnd_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftLk_SpecialN_UnsetFv14(gobj);
        updateParts(gobj);
        st::ft_8008A2BC(gobj);
    } else {
        updateParts(gobj);
    }
}

pub fn ftLk_SpecialAirNStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialAirNStart_Anim");
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        if (*fp).cmd_vars[2] != 0 && (*fp).mv.lk.specialn.x0.x == 0.0 {
            (*fp).mv.lk.specialn.x0.x = 1.0;
            (*fp).cmd_vars[2] = 0;
        }
        if !isDrawn(gobj) {
            animate(gobj);
            if st::ftAnim_IsFramesRemaining(gobj) == 0 {
                st::Fighter_ChangeMotionState(gobj, ftLk_MS_SpecialAirNLoop as i32, mf, 0.0, 1.0, 0.0, core::ptr::null_mut());
                set_damage_cb(gobj);
                st::ftAnim_8006EBA4(gobj);
            }
        } else {
            ftLk_SpecialN_UnsetArrow(gobj);
            ftLk_SpecialN_UnsetFv14(gobj);
            air_abort(gobj, a);
        }
    }
}

pub fn ftLk_SpecialAirNLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialAirNLoop_Anim");
    animate(gobj);
}

pub fn ftLk_SpecialAirNEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialAirNEnd_Anim");
    unsafe {
        let a = da(fpp(gobj));
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ftLk_SpecialN_UnsetArrow(gobj);
            ftLk_SpecialN_UnsetFv14(gobj);
            updateParts(gobj);
            air_abort(gobj, a);
        } else {
            updateParts(gobj);
        }
    }
}

/// Shared body of the Start IASA callbacks (the C duplicates it with a different end MSID).
fn start_iasa(gobj: *mut HSD_GObj, end_msid: i32) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        if (*fp).mv.lk.specialn.x0.x != 0.0 {
            (*fp).mv.lk.specialn.x0.y += 1.0;
            if (*fp).mv.lk.specialn.x0.y > a.x0 {
                (*fp).mv.lk.specialn.x0.y = a.x0;
            }
            if (*fp).input.held_buttons[0] & (HSD_PAD_B as u32) == 0 {
                st::Fighter_ChangeMotionState(gobj, end_msid, mf, 0.0, 1.0, 0.0, core::ptr::null_mut());
                set_damage_cb(gobj);
            }
        }
    }
}

pub fn ftLk_SpecialNStart_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialNStart_IASA");
    start_iasa(gobj, ftLk_MS_SpecialNEnd as i32);
}

/// `doLoopIASA` (static).
fn doLoopIASA(gobj: *mut HSD_GObj, msid: i32) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        (*fp).mv.lk.specialn.x0.y = a.x0;
        if (*fp).input.held_buttons[0] & (HSD_PAD_B as u32) == 0 {
            st::Fighter_ChangeMotionState(gobj, msid, mf, 0.0, 1.0, 0.0, core::ptr::null_mut());
            set_damage_cb(gobj);
        }
    }
}

pub fn ftLk_SpecialNLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialNLoop_IASA");
    doLoopIASA(gobj, ftLk_MS_SpecialNEnd as i32);
}

pub fn ftLk_SpecialNEnd_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLk_SpecialNEnd_IASA");}

pub fn ftLk_SpecialAirNStart_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialAirNStart_IASA");
    start_iasa(gobj, ftLk_MS_SpecialAirNEnd as i32);
}

pub fn ftLk_SpecialAirNLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialAirNLoop_IASA");
    doLoopIASA(gobj, ftLk_MS_SpecialAirNEnd as i32);
}

pub fn ftLk_SpecialAirNEnd_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLk_SpecialAirNEnd_IASA");}

/// `doEndColl` (static inline): fire the arrow when the script's cmd_vars[1] pulse arrives.
fn doEndColl(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        if (*fp).cmd_vars[1] == 1 && !(*fp).u.lk.arrow_gobj.is_null() {
            let mut rpos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let mut lpos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            (*fp).cmd_vars[1] = 0;
            ext::lb_8000B1CC(bone_joint(fp, FtPart_RThumbNb), None, &mut rpos);
            ext::lb_8000B1CC(bone_joint(fp, FtPart_LThumbNb), None, &mut lpos);
            lpos.z = 0.0;
            rpos.z = 0.0;
            let item_gobj = (*fp).item_gobj;
            ext::itLinkArrow_802A850C((*fp).u.lk.arrow_gobj, &rpos, &lpos, DEG5_RAD, (*fp).mv.lk.specialn.x0.y, a.x0);
            ftLk_SpecialN_UnsetArrow(gobj);
            (*fp).item_gobj = item_gobj;
            melee_fighter::tu::ftpickupitem::ftpickupitem_80094818(gobj, false);
        }
    }
}

/// `doColl` (static): ground to air.
fn doColl(gobj: *mut HSD_GObj, msid: i32) {
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80082708(gobj) == 0 {
            st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, msid, coll_mf);
            set_damage_cb(gobj);
            st::ftAnim_8006EBA4(gobj);
        }
    }
}

/// `doAirColl` (static): air to ground.
fn doAirColl(gobj: *mut HSD_GObj, msid: i32) {
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80081D0C(gobj) == GA_Air as i32 {
            st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, msid, coll_mf);
            set_damage_cb(gobj);
            st::ftAnim_8006EBA4(gobj);
        }
    }
}

pub fn ftLk_SpecialNStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialNStart_Coll");
    doColl(gobj, ftLk_MS_SpecialAirNStart as i32);
}

pub fn ftLk_SpecialNLoop_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialNLoop_Coll");
    doColl(gobj, ftLk_MS_SpecialAirNLoop as i32);
}

pub fn ftLk_SpecialNEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialNEnd_Coll");
    unsafe {
        let fp = fpp(gobj);
        doEndColl(gobj);
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftLk_MS_SpecialAirNEnd as i32, coll_mf);
            set_damage_cb(gobj);
            st::ftAnim_8006EBA4(gobj);
        }
    }
}

pub fn ftLk_SpecialAirNStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialAirNStart_Coll");
    doAirColl(gobj, ftLk_MS_SpecialNStart as i32);
}

pub fn ftLk_SpecialAirNLoop_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialAirNLoop_Coll");
    doAirColl(gobj, ftLk_MS_SpecialNLoop as i32);
}

pub fn ftLk_SpecialAirNEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialAirNEnd_Coll");
    unsafe {
        let fp = fpp(gobj);
        doEndColl(gobj);
        if st::ft_80081D0C(gobj) == GA_Air as i32 {
            st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftLk_MS_SpecialNEnd as i32, coll_mf);
            set_damage_cb(gobj);
            st::ftAnim_8006EBA4(gobj);
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftLk_SpecialNStart_Phys` (src/melee/ft/kinds/ftLink/ftlinkspecialn.c).
pub fn ftLk_SpecialNStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialNStart_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftLk_SpecialNLoop_Phys` (src/melee/ft/kinds/ftLink/ftlinkspecialn.c).
pub fn ftLk_SpecialNLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialNLoop_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftLk_SpecialNEnd_Phys` (src/melee/ft/kinds/ftLink/ftlinkspecialn.c).
pub fn ftLk_SpecialNEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialNEnd_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftLk_SpecialAirNStart_Phys` (src/melee/ft/kinds/ftLink/ftlinkspecialn.c).
pub fn ftLk_SpecialAirNStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialAirNStart_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftLk_SpecialAirNLoop_Phys` (src/melee/ft/kinds/ftLink/ftlinkspecialn.c).
pub fn ftLk_SpecialAirNLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialAirNLoop_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftLk_SpecialAirNEnd_Phys` (src/melee/ft/kinds/ftLink/ftlinkspecialn.c).
pub fn ftLk_SpecialAirNEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialAirNEnd_Phys");
    st::ft_80084EEC(gobj);
}

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftLk_SpecialN_IsActiveAnd2071b6_fp(fp: &Fighter) -> bool {
    melee_core::ct!("ftLk_SpecialN_IsActiveAnd2071b6_fp");
    let m = fp.motion_id as u32;
    let active = [
        ftLk_MS_SpecialNStart,
        ftLk_MS_SpecialNLoop,
        ftLk_MS_SpecialNEnd,
        ftLk_MS_SpecialAirNStart,
        ftLk_MS_SpecialAirNLoop,
        ftLk_MS_SpecialAirNEnd,
    ];
    if active.iter().any(|&s| s as u32 == m) && unsafe { fp.x2070.anon_0.x2071_b6() } == 0 {
        return false;
    }
    true
}
