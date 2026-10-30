//! src/melee/ft/kinds/ftZelda/ftzeldaspecials.c: Zelda's side B (Stop, 8013B638..8013C2F8).
//! The functions not ported elsewhere are checked bit for bit against the C by a state oracle
//! (tests/tu_ftzeldaspecials.rs over gen/turef/tu_ftzeldaspecials_ref.c, shipping contraction).
//! Fused sites (tools/rust-port/fma_sites.py): `temp_f2 * facing_dir + x` in the four `*_Anim`
//! functions that spawn the Din's Fire item (`sp.x = (x20 * facing_dir) + sp.x`), written with
//! `melee_core::fp::fmadds`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, dead_code)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (round 2/12): re-exported under this TU.
pub use crate::chars::r12::ftZelda::ftzeldaspecials::{
    ftZd_SpecialAirSEnd_IASA, ftZd_SpecialAirSEnd_Coll, ftZd_SpecialAirSLoop_Coll, ftZd_SpecialAirSStart_Coll,
    ftZd_SpecialSEnd_Coll, ftZd_SpecialSLoop_Coll, ftZd_SpecialSStart_Coll,
};
pub use crate::chars::r2::ftZelda::ftzeldaspecials::{ftZd_SpecialSEnd_Phys, ftZd_SpecialSLoop_Phys};
pub use crate::chars::ftZelda::ftzeldaspecials::{
    ftZd_SpecialAirSEnd_Phys, ftZd_SpecialAirSLoop_Phys, ftZd_SpecialAirSStart_Phys,
};

/// Calls into TUs that are not ported yet. Each is a settable hook; an unset hook panics.
pub mod ext {
    use core::ffi::c_void;
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
    // `efSync_Spawn(gfx_id, gobj, joint)` (ef/efsync.c, variadic; the port passes the one joint).
    pub static mut EFSYNC_SPAWN: Option<fn(i32, *mut HSD_GObj, *mut c_void) -> *mut c_void> = None;
    #[allow(non_snake_case)]
    pub fn efSync_Spawn(gfx_id: i32, gobj: *mut HSD_GObj, joint: *mut c_void) -> *mut c_void {
        match unsafe { EFSYNC_SPAWN } {
            Some(f) => f(gfx_id, gobj, joint),
            None => { unsafe { melee_world::tu::efsync::efSync_Spawn(gfx_id as _, gobj as _, &mut *(joint as *mut _)) as _ } },
        }
    }
    // `lb_8000B1CC(jobj, pos0, pos1)` (lb/lb_00B0.c).
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(j: *mut HSD_JObj, pos0: *const Vec3, pos1: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(j, pos0, pos1),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(j as _, pos0 as _, pos1 as _); } return; },
                }
            }
    // `it_802C3BAC(gobj, vec, dir, x)` (it/kinds/itzeldadinfire.c): the Din's Fire item, or NULL.
    pub static mut IT_802C3BAC: Option<fn(*mut HSD_GObj, *mut Vec3, f32, f32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802C3BAC(gobj: *mut HSD_GObj, vec: *mut Vec3, dir: f32, x: f32) -> *mut HSD_GObj {
                match unsafe { IT_802C3BAC } {
                    Some(f) => f(gobj, vec, dir, x),
                    None => { return unsafe { melee_items::tu::itzeldadinfire::it_802C3BAC(gobj as _, vec as _, dir as _, x as _) as _ }; },
                }
            }
    // `itZeldaDinFire_GetOwner(item)` (it/kinds/itzeldadinfire.c).
    pub static mut ITZELDADINFIRE_GETOWNER: Option<fn(*mut HSD_GObj) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn itZeldaDinFire_GetOwner(item: *mut HSD_GObj) -> *mut HSD_GObj {
                match unsafe { ITZELDADINFIRE_GETOWNER } {
                    Some(f) => f(item),
                    None => { return unsafe { melee_items::tu::itzeldadinfire::itZeldaDinFire_GetOwner(item as _) as _ }; },
                }
            }
    // `ftPartSetRotX(fp, part_idx, rot)` (ft/ftparts.c): its port needs the unported JObj setter.
    pub static mut FTPARTSETROTX: Option<fn(*mut Fighter, i32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn ftPartSetRotX(fp: *mut Fighter, part_idx: i32, rot: f32) {
        match unsafe { FTPARTSETROTX } {
            Some(f) => f(fp, part_idx, rot),
            None => { unsafe { let _ = melee_fighter::tu::ftparts::ftPartSetRotX(&*(fp as *const _), part_idx as _, rot as _); } },
        }
    }
}

/// `ftZelda_DatAttrs` of a fighter (`fp->dat_attrs`).
#[inline(always)]
unsafe fn da_of(fp: *mut Fighter) -> *mut ftZelda_DatAttrs {
    (*fp).dat_attrs as *mut ftZelda_DatAttrs
}

/// `GET_FIGHTER(gobj)`.
#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `fp->parts[part].joint`.
#[inline(always)]
unsafe fn joint(fp: *mut Fighter, part: usize) -> *mut core::ffi::c_void {
    (*(*fp).parts.add(part)).joint as *mut core::ffi::c_void
}

/// `ftZd_Init_801393AC` (ft/kinds/ftZelda/ftzelda.c): not ported yet (a callback address only here).
pub unsafe extern "C" fn ftZd_Init_801393AC(_gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_Init_801393AC");
    { unsafe { let _ = crate::tu::ftzelda::ftZd_Init_801393AC(_gobj as _); } return; };
}

/// `ftZd_MF_SpecialSStart_Coll` is the coll TU's (r12); the Enter/Anim functions here use no flags.
/// The Din's Fire spawn shared by the four `*_Anim` functions that open the item window (`SStart`
/// and `AirSStart` use `x20`/`x24`/`fighter->parts[89]`, the loop variants the same).
unsafe fn spawn_din_fire_on_window(gobj: *mut HSD_GObj, fp: *mut Fighter, sa: *mut ftZelda_DatAttrs) {
    let mut sp: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC((*(*fp).parts.add(89)).joint, core::ptr::null(), &mut sp);
    sp.z = 0.0;
    let temp_f2 = (*sa).x20;
    sp.x = fmadds(temp_f2, (*fp).facing_dir, sp.x);
    sp.y += (*sa).x24;
    let item = ext::it_802C3BAC(gobj, &mut sp, (*fp).facing_dir, temp_f2);
    (*fp).u.zd.x222C = item;
    if !item.is_null() {
        (*fp).death2_cb = Some(ftZd_Init_801393AC);
        (*fp).take_dmg_cb = Some(ftZd_Init_801393AC);
    }
    ext::efSync_Spawn(1275, gobj, joint(fp, 76));
}

/// `ftZd_SpecialS_Enter` (343, Zelda's side B on the ground).
pub fn ftZd_SpecialS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialS_Enter");
    unsafe {
        let temp_f1: f32 = 0.0;
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(gobj, 343, 0, temp_f1, 1.0, temp_f1, core::ptr::null_mut());
        st::Fighter_ClearCmdVars(&mut *fp);
        let attributes = da_of(fp);
        (*fp).mv.zd.specials.x0 = (*attributes).x10;
        (*fp).mv.zd.specials.x4 = (*attributes).x14;
        (*fp).mv.zd.specials.x8 = (*attributes).x18;
        (*fp).u.zd.x222C = core::ptr::null_mut();
        (*fp).mv.zd.specials.xC = (*attributes).x1C;
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
        st::ftAnim_8006EBA4(gobj);
    }
}

/// `ftZd_SpecialAirS_Enter` (346, the aerial side B: stops vertical velocity).
pub fn ftZd_SpecialAirS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialAirS_Enter");
    unsafe {
        let temp_f1: f32 = 0.0;
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(gobj, 346, 0, temp_f1, 1.0, temp_f1, core::ptr::null_mut());
        st::Fighter_ClearCmdVars(&mut *fp);
        (*fp).self_vel.y = 0.0;
        let attributes = da_of(fp);
        (*fp).mv.zd.specials.x0 = (*attributes).x10;
        (*fp).mv.zd.specials.x4 = (*attributes).x14;
        (*fp).mv.zd.specials.x8 = (*attributes).x18;
        (*fp).u.zd.x222C = core::ptr::null_mut();
        (*fp).mv.zd.specials.xC = (*attributes).x1C;
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
        st::ftAnim_8006EBA4(gobj);
    }
}

/// `ftZd_SpecialSStart_Anim` (AS 343 window: spawn the item on cmd_vars[0] == 1, then 344).
pub fn ftZd_SpecialSStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialSStart_Anim");
    unsafe {
        let fp = fpp(gobj);
        let attributes = da_of(fp);
        if (*fp).cmd_vars[0] == 1 && (*fp).u.zd.x222C.is_null() {
            (*fp).cmd_vars[0] = 0;
            spawn_din_fire_on_window(gobj, fp, attributes);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let temp_f1: f32 = 0.0;
            st::Fighter_ChangeMotionState(gobj, 344, 0, temp_f1, 1.0, temp_f1, core::ptr::null_mut());
            (*fp).x1968_jumpsUsed = crate::chars::caps_max_jumps(&*fp) as u8;
        }
    }
}

/// `ftZd_SpecialSLoop_Anim` (AS 344: the item's owner check, then 345 once the pulses are spent).
pub fn ftZd_SpecialSLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialSLoop_Anim");
    unsafe {
        let fp = fpp(gobj);
        let attributes = da_of(fp);
        if (*fp).cmd_vars[0] == 1 && (*fp).u.zd.x222C.is_null() {
            (*fp).cmd_vars[0] = 0;
            spawn_din_fire_on_window(gobj, fp, attributes);
        }
        if (*fp).mv.zd.specials.x0 != 0 {
            (*fp).mv.zd.specials.x0 -= 1;
        }
        if (*fp).mv.zd.specials.x4 != 0 {
            (*fp).mv.zd.specials.x4 -= 1;
        }
        if (*fp).u.zd.x222C.is_null() {
            if (*fp).mv.zd.specials.x0 <= 0 && (*fp).mv.zd.specials.x4 <= 0 {
                let temp_f1: f32 = 0.0;
                st::Fighter_ChangeMotionState(gobj, 345, 0, temp_f1, 1.0, temp_f1, core::ptr::null_mut());
            }
        } else {
            let owner = ext::itZeldaDinFire_GetOwner((*fp).u.zd.x222C);
            if owner != gobj {
                (*fp).u.zd.x222C = core::ptr::null_mut();
            }
        }
    }
}

/// `ftZd_SpecialSEnd_Anim` (AS 345: reset the side B state, then `ft_8008A2BC` when the anim ends).
pub fn ftZd_SpecialSEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialSEnd_Anim");
    unsafe {
        let fp = fpp(gobj);
        ft_part_set_rot_x(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = fpp(gobj);
            let attributes = da_of(fp);
            (*fp).mv.zd.specials.x0 = (*attributes).x10;
            (*fp).mv.zd.specials.x4 = (*attributes).x14;
            (*fp).mv.zd.specials.x8 = (*attributes).x18;
            (*fp).u.zd.x222C = core::ptr::null_mut();
            (*fp).mv.zd.specials.xC = (*attributes).x1C;
            (*fp).death2_cb = None;
            (*fp).take_dmg_cb = None;
            st::ft_8008A2BC(gobj);
        }
    }
}

/// `ftPartSetRotX(fp, 0, 0)` at the top of the two `*_Anim` end functions.
#[inline(always)]
unsafe fn ft_part_set_rot_x(fp: *mut Fighter) {
    ext::ftPartSetRotX(fp, 0, 0.0);
}

/// `ftZd_SpecialAirSStart_Anim` (AS 346 window, the aerial version of `SStart_Anim`; ends on 347).
pub fn ftZd_SpecialAirSStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialAirSStart_Anim");
    unsafe {
        let fp = fpp(gobj);
        let attributes = da_of(fp);
        if (*fp).cmd_vars[0] == 1 && (*fp).u.zd.x222C.is_null() {
            (*fp).cmd_vars[0] = 0;
            spawn_din_fire_on_window(gobj, fp, attributes);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let temp_f1: f32 = 0.0;
            st::Fighter_ChangeMotionState(gobj, 347, 0, temp_f1, 1.0, temp_f1, core::ptr::null_mut());
            (*fp).x1968_jumpsUsed = crate::chars::caps_max_jumps(&*fp) as u8;
        }
    }
}

/// `ftZd_SpecialAirSLoop_Anim` (AS 347: like `SLoop_Anim`, ends on 348 instead of 345).
pub fn ftZd_SpecialAirSLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialAirSLoop_Anim");
    unsafe {
        let fp = fpp(gobj);
        let attributes = da_of(fp);
        if (*fp).cmd_vars[0] == 1 && (*fp).u.zd.x222C.is_null() {
            (*fp).cmd_vars[0] = 0;
            spawn_din_fire_on_window(gobj, fp, attributes);
        }
        if (*fp).mv.zd.specials.x0 != 0 {
            (*fp).mv.zd.specials.x0 -= 1;
        }
        if (*fp).mv.zd.specials.x4 != 0 {
            (*fp).mv.zd.specials.x4 -= 1;
        }
        if (*fp).u.zd.x222C.is_null() {
            if (*fp).mv.zd.specials.x0 <= 0 && (*fp).mv.zd.specials.x4 <= 0 {
                let temp_f1: f32 = 0.0;
                st::Fighter_ChangeMotionState(gobj, 348, 0, temp_f1, 1.0, temp_f1, core::ptr::null_mut());
            }
        } else {
            let owner = ext::itZeldaDinFire_GetOwner((*fp).u.zd.x222C);
            if owner != gobj {
                (*fp).u.zd.x222C = core::ptr::null_mut();
            }
        }
    }
}

/// `ftZd_SpecialAirSEnd_Anim` (AS 348 end: `ftCo_Fall_Enter`, or the landing lag branch).
pub fn ftZd_SpecialAirSEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialAirSEnd_Anim");
    unsafe {
        let fp = fpp(gobj);
        ft_part_set_rot_x(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = fpp(gobj);
            let sa = da_of(fp);
            (*fp).mv.zd.specials.x0 = (*sa).x10;
            (*fp).mv.zd.specials.x4 = (*sa).x14;
            (*fp).mv.zd.specials.x8 = (*sa).x18;
            (*fp).u.zd.x222C = core::ptr::null_mut();
            (*fp).mv.zd.specials.xC = (*sa).x1C;
            (*fp).death2_cb = None;
            (*fp).take_dmg_cb = None;
            if (*sa).x34 == 0.0 {
                st::ftCo_Fall_Enter(gobj);
            } else {
                st::ftCo_80096900(gobj, 1, 0, 1, 1.0, (*sa).x34);
            }
        }
    }
}

/// `ftZd_SpecialSStart_IASA` (empty).
pub fn ftZd_SpecialSStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftZd_SpecialSStart_IASA");}

/// `ftZd_SpecialSLoop_IASA`: count down xC; release to 345 when spent and B is not held.
pub fn ftZd_SpecialSLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialSLoop_IASA");
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.zd.specials.xC -= 1;
        let var_r0 = if (*fp).mv.zd.specials.xC <= 0 {
            (*fp).mv.zd.specials.xC = 0;
            1
        } else {
            0
        };
        if var_r0 == 1 && ((*fp).input.held_buttons[0] & 512) == 0 {
            let temp_f1: f32 = 0.0;
            st::Fighter_ChangeMotionState(gobj, 345, 0, temp_f1, 1.0, temp_f1, core::ptr::null_mut());
        }
    }
}

/// `ftZd_SpecialSEnd_IASA` (empty).
pub fn ftZd_SpecialSEnd_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftZd_SpecialSEnd_IASA");}

/// `ftZd_SpecialAirSStart_IASA` (empty).
pub fn ftZd_SpecialAirSStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftZd_SpecialAirSStart_IASA");}

/// `ftZd_SpecialAirSLoop_IASA`: as `SLoop_IASA`, releasing to 348.
pub fn ftZd_SpecialAirSLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialAirSLoop_IASA");
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.zd.specials.xC -= 1;
        let var_r0 = if (*fp).mv.zd.specials.xC <= 0 {
            (*fp).mv.zd.specials.xC = 0;
            1
        } else {
            0
        };
        if var_r0 == 1 && ((*fp).input.held_buttons[0] & 512) == 0 {
            let temp_f1: f32 = 0.0;
            st::Fighter_ChangeMotionState(gobj, 348, 0, temp_f1, 1.0, temp_f1, core::ptr::null_mut());
        }
    }
}

/// `ftZd_SpecialSStart_Phys`: count down x8, then the ground physics.
pub fn ftZd_SpecialSStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialSStart_Phys");
    unsafe {
        let fp = fpp(gobj);
        let state_var = (*fp).mv.zd.specials.x8;
        if state_var != 0 {
            (*fp).mv.zd.specials.x8 -= 1;
        }
        st::ft_80084F3C(gobj);
    }
}
