//! src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c: Kirby's Shadow Ball (Mewtwo copy ability).
//! No fused sites (tools/rust-port/fma_sites.py). The charge-ratio sfx ladder and the recoil are
//! pure and checked against the C by tests/tu_ftkirbyspecialmewtwo.rs; the states have a smoke test.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use crate::tu::ftkirbyspecialn::{change_ms, fpp, G};
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (rounds 2, 11): re-exported under this TU.



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
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    type G = *mut HSD_GObj;
    pub use crate::tu::ftcaptainspecialn::ext::{efSync_Spawn, EFSYNC_SPAWN};
    pub use crate::tu::ftkirbyspecialn::ext::{lb_8000B1CC, LB_8000B1CC};
    pub static mut IT_802C573C: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C573C(item: *mut Item_GObj)  {
                match unsafe { IT_802C573C } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itmewtwoshadowball::it_802C573C(item as _); } return; },
                }
            }
    pub static mut IT_802C53F0: Option<fn(*mut Item_GObj, *const Vec3, f32, f32, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C53F0(item: *mut Item_GObj, pos: *const Vec3, angle: f32, charge: f32, max: f32)  {
                match unsafe { IT_802C53F0 } {
                    Some(f) => f(item, pos, angle, charge, max),
                    None => { unsafe { melee_items::tu::itmewtwoshadowball::it_802C53F0(item as _, pos as _, angle as _, charge as _, max as _); } return; },
                }
            }
    pub static mut IT_802C5000: Option<fn(G, *const Vec3, i32, i32, f32) -> *mut Item_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802C5000(g: G, pos: *const Vec3, part: i32, kind: i32, dir: f32) -> *mut Item_GObj {
                match unsafe { IT_802C5000 } {
                    Some(f) => f(g, pos, part, kind, dir),
                    None => { return unsafe { melee_items::tu::itmewtwoshadowball::it_802C5000(g as _, pos as _, part as _, kind as _, dir as _) as _ }; },
                }
            }
    pub static mut FTCO_800BFFAC: Option<fn(*mut Fighter)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800BFFAC(fp: *mut Fighter) {
        match unsafe { FTCO_800BFFAC } {
            Some(f) => f(fp),
            None => { unsafe { melee_fighter::tu::ftcolanim::ftco_800bffac(&mut *(fp as *mut _)); } },
        }
    }
    pub static mut FTCO_800BFFD0: Option<fn(*mut Fighter, i32, i32) -> bool> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800BFFD0(fp: *mut Fighter, a: i32, b: i32) -> bool {
        match unsafe { FTCO_800BFFD0 } {
            Some(f) => f(fp, a, b),
            None => { unsafe { melee_fighter::tu::ftcolanim::ftco_800bffd0(&mut *(fp as *mut _), a as _, b as _) as _ } },
        }
    }
    pub static mut FT_80088510: Option<fn(*mut Fighter, u32, i32, i32)> = None;
    #[allow(non_snake_case)]
    pub fn ft_80088510(fp: *mut Fighter, sfx: u32, vol: i32, pan: i32) {
        match unsafe { FT_80088510 } {
            Some(f) => f(fp, sfx, vol, pan),
            None => { unsafe { let _ = melee_fighter::ft0881::ft_80088510(&mut *(fp as *mut _), sfx as _, vol as _, pan as _); } },
        }
    }
    pub static mut FTCO_8009917C: Option<fn(G) -> bool> = None;
            #[allow(non_snake_case)]
            pub fn ftCo_8009917C(g: G) -> bool {
                match unsafe { FTCO_8009917C } {
                    Some(f) => f(g),
                    None => { return unsafe { melee_fighter::tu::ftCo_Escape::ftCo_8009917C(g as _) as _ }; },
                }
            }
}

/// `ftMt_MF_SpecialNLoop_Coll` (ftMewtwo/forward.h).
pub const ftMt_MF_SpecialNLoop_Coll: u32 = 0xC4C5280;
const MF_LOOPFULL: u32 =
    (Ft_MF_SkipMatAnim | Ft_MF_KeepSfx | Ft_MF_UpdateCmd | Ft_MF_SkipItemVis | Ft_MF_Unk19) as u32;

/// `ftKb_Init_803B7570` / `ftKb_Init_803B757C`: the full-charge effect offset.
pub const ftKb_Init_803B7570: Vec3 = Vec3 { x: 0.0, y: 4.0, z: 0.0 };
pub const ftKb_Init_803B757C: Vec3 = Vec3 { x: 0.0, y: 4.0, z: 0.0 };
/// `ftKb_Init_803CB828`: charge-step sfx ids.
pub static ftKb_Init_803CB828: [u32; 4] = [0x00030DB9, 0x00030DBC, 0x00030DBF, 0x00030DC2];

#[inline(always)]
unsafe fn da_of(fp: *mut Fighter) -> *const ftKb_DatAttrs {
    (*fp).dat_attrs as *const ftKb_DatAttrs
}

#[inline(always)]
unsafe fn set_cbs(gobj: G) {
    crate::tu::ftkirbyspecialn::ftKb_SpecialN_set_cbs(gobj)
}

/// Charge readout for the HUD. Returns -1 when there is no shadow ball.
pub fn ftKb_SpecialNMt_80106F44(gobj: G, out_charge: &mut i32, out_max: &mut i32) -> i32 {
    melee_core::ct!("ftKb_SpecialNMt_80106F44");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            let da = &*da_of(fp);
            if (*fp).u.kb.x98.is_null() {
                return -1;
            }
            *out_charge = (*fp).u.kb.x9C;
            *out_max = da.specialn_mt_charge_time as i32;
            return 0;
        }
        -1
    }
}

fn is_mt_special_n_motion(msid: i32) -> bool {
    matches!(
        msid,
        m if m == ftKb_MS_MtSpecialNStart as i32
            || m == ftKb_MS_MtSpecialNLoop as i32
            || m == ftKb_MS_MtSpecialNLoopFull as i32
            || m == ftKb_MS_MtSpecialNCancel as i32
            || m == ftKb_MS_MtSpecialNEnd as i32
            || m == ftKb_MS_MtSpecialAirNStart as i32
            || m == ftKb_MS_MtSpecialAirNLoop as i32
            || m == ftKb_MS_MtSpecialAirNLoopFull as i32
            || m == ftKb_MS_MtSpecialAirNCancel as i32
            || m == ftKb_MS_MtSpecialAirNEnd as i32
    )
}

pub fn ftKb_SpecialNMt_80106F9C(gobj: G) -> bool {
    melee_core::ct!("ftKb_SpecialNMt_80106F9C");
    unsafe {
        if !gobj.is_null() {
            let fp = (*gobj).user_data as *mut Fighter;
            if is_mt_special_n_motion((*fp).motion_id as i32) {
                return (*fp).x2070.anon_0.x2071_b6() != 0;
            }
            return true;
        }
        true
    }
}

pub fn ftKb_SpecialNMt_80106FEC(gobj: G) -> bool {
    melee_core::ct!("ftKb_SpecialNMt_80106FEC");
    unsafe {
        if !gobj.is_null() {
            let fp = (*gobj).user_data as *mut Fighter;
            let m = (*fp).motion_id as i32;
            // Cancel (and AirCancel) are not in this list, unlike 80106F9C's.
            return !(m == ftKb_MS_MtSpecialNStart as i32
                || m == ftKb_MS_MtSpecialNLoop as i32
                || m == ftKb_MS_MtSpecialNLoopFull as i32
                || m == ftKb_MS_MtSpecialNEnd as i32
                || m == ftKb_MS_MtSpecialAirNStart as i32
                || m == ftKb_MS_MtSpecialAirNLoop as i32
                || m == ftKb_MS_MtSpecialAirNLoopFull as i32
                || m == ftKb_MS_MtSpecialAirNEnd as i32);
        }
        true
    }
}

/// Clears the Mewtwo copy ability state and effects.
pub fn ftKb_SpecialNMt_80107040(gobj: G) {
    melee_core::ct!("ftKb_SpecialNMt_80107040");
    unsafe {
        if gobj.is_null() {
            return;
        }
        let fp = fpp(gobj);
        if !(*fp).u.kb.x98.is_null() {
            (*fp).u.kb.x98 = core::ptr::null_mut();
        }
        let fp = fpp(gobj);
        st::efLib_DestroyAll(gobj);
        (*fp).u.kb.xA0 = core::ptr::null_mut();
    }
}

/// Releases the shadow ball item and destroys the effects (the tail every cancel path shares).
unsafe fn release_ball(gobj: G) {
    if !gobj.is_null() {
        let fp = fpp(gobj);
        if !(*fp).u.kb.x98.is_null() {
            ext::it_802C573C((*fp).u.kb.x98);
            (*fp).u.kb.x98 = core::ptr::null_mut();
        }
        if !gobj.is_null() {
            let fp2 = fpp(gobj);
            st::efLib_DestroyAll(gobj);
            (*fp2).u.kb.xA0 = core::ptr::null_mut();
        }
    }
}

/// Full cleanup of the Mewtwo copy ability state.
pub fn ftKb_SpecialNMt_8010709C(gobj: G) {
    melee_core::ct!("ftKb_SpecialNMt_8010709C");
    unsafe {
        if gobj.is_null() {
            return;
        }
        let fp2 = fpp(gobj);
        release_ball(gobj);
        (*fp2).u.kb.x9C = 0;
        ext::ftCo_800BFFAC(fp2);
        st::efLib_DestroyAll(gobj);
    }
}

/// Mewtwo copy cleanup with charge check.
pub fn ftKb_SpecialNMt_80107130(gobj: G) {
    melee_core::ct!("ftKb_SpecialNMt_80107130");
    unsafe {
        if gobj.is_null() {
            return;
        }
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        let fp2 = fp;
        release_ball(gobj);
        if (*fp2).u.kb.x9C as f32 == da.specialn_mt_charge_time {
            return;
        }
        (*fp2).u.kb.x9C = 0;
        ext::ftCo_800BFFAC(fp2);
        st::efLib_DestroyAll(gobj);
    }
}

/// `ftKb_SpecialNMt_SetRecoil` (static inline): launch recoil proportional to the charge.
pub fn ftKb_SpecialNMt_SetRecoil(gobj: G) {
    melee_core::ct!("ftKb_SpecialNMt_SetRecoil");
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        if (*fp).motion_id == ftKb_MS_MtSpecialAirNEnd as FtMotionId || (*fp).ground_or_air == GA_Air {
            (*fp).self_vel.x = (*fp).facing_dir * (da.specialn_mt_air_horizontal_momentum * (*fp).u.kb.x9C as f32);
        }
        if (*fp).motion_id == ftKb_MS_MtSpecialNEnd as FtMotionId || (*fp).ground_or_air == GA_Ground {
            (*fp).gr_vel = (*fp).facing_dir * (da.specialn_mt_ground_horizontal_momentum * (*fp).u.kb.x9C as f32);
        }
    }
}

/// Fires the shadow ball (cmd_vars[1] == 1 on the End states).
pub fn ftKb_SpecialNMt_801071FC(gobj: G) {
    melee_core::ct!("ftKb_SpecialNMt_801071FC");
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        if (*fp).cmd_vars[1] == 1 && !(*fp).u.kb.x98.is_null() {
            (*fp).cmd_vars[1] = 2;
            let sp2c = Vec3 { x: 0.0, y: 0.0, z: 2.0 };
            let mut sp38 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_RHandN as usize)).joint as *mut _, &sp2c, &mut sp38);
            sp38.z = 0.0;
            let saved_item = (*fp).item_gobj;
            let launch_angle: f64 = if (*fp).facing_dir == 1.0 { 0.0 } else { core::f64::consts::PI };
            ext::it_802C53F0((*fp).u.kb.x98, &sp38, launch_angle as f32, (*fp).u.kb.x9C as u32 as f32, da.specialn_mt_charge_time);
            ftKb_SpecialNMt_SetRecoil(gobj);
            (*fp).u.kb.x9C = 0;
            ftKb_SpecialNMt_80107040(gobj);
            (*fp).item_gobj = saved_item;
            if (*fp).u.kb.x9C as f32 == da.specialn_mt_charge_time {
                fh::play_sfx(&mut *fp, 0x30DB6, 0x7F, 0x40);
                return;
            }
            fh::play_sfx(&mut *fp, 0x30DB3, 0x7F, 0x40);
        }
    }
}

/// Charge-step sfx ladder (cmd_vars[2] != 0 while charging).
pub fn ftKb_SpecialNMt_80107410(gobj: G) {
    melee_core::ct!("ftKb_SpecialNMt_80107410");
    unsafe {
        let sfx_table = &ftKb_Init_803CB828;
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        if (*fp).cmd_vars[2] != 0 {
            let charge = (*fp).u.kb.x9C;
            let charge_ratio: f32 = if charge != 0 { charge as f32 / da.specialn_mt_charge_time } else { 0.0 };
            let prev_ratio = (*fp).mv.kb.specialhi.x10.f;
            if prev_ratio > 0.0 {
                if charge_ratio > 0.75 && prev_ratio <= 0.75 {
                    (*fp).mv.kb.specialhi.x10.f = charge_ratio;
                    ext::ft_80088510(fp, sfx_table[3], 0x7F, 0x40);
                    return;
                }
                if charge_ratio > 0.5 && prev_ratio <= 0.5 {
                    (*fp).mv.kb.specialhi.x10.f = charge_ratio;
                    ext::ft_80088510(fp, sfx_table[2], 0x7F, 0x40);
                    return;
                }
                if charge_ratio > 0.25 && prev_ratio <= 0.25 {
                    (*fp).mv.kb.specialhi.x10.f = charge_ratio;
                    ext::ft_80088510(fp, sfx_table[1], 0x7F, 0x40);
                    return;
                }
                if charge_ratio > 0.0 && prev_ratio <= 0.0 {
                    (*fp).mv.kb.specialhi.x10.f = charge_ratio;
                    ext::ft_80088510(fp, sfx_table[0], 0x7F, 0x40);
                }
            } else {
                (*fp).mv.kb.specialhi.x10.f = 0.0001;
                ext::ft_80088510(fp, sfx_table[0], 0x7F, 0x40);
            }
        }
    }
}

/// Ground Shadow Ball entry (from the copy ability's neutral B).
pub fn ftKb_SpecialNMt_80107568(gobj: G) {
    melee_core::ct!("ftKb_SpecialNMt_80107568");
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        change_ms(gobj, ftKb_MS_MtSpecialNStart as i32, 0);
        st::Fighter_ClearCmdVars(&mut *fp);
        st::ftCommon_8007D7FC(&mut *fp);
        (*fp).self_vel.y = 0.0;
        set_cbs(gobj);
        (*fp).mv.kb.specialhi.x0 = 0;
        (*fp).mv.kb.specialhi.x4 = 0;
        (*fp).mv.kb.specialhi.xC = if (*fp).u.kb.x9C == 0 { da.specialn_mt_frames_to_transition } else { 0 };
        (*fp).mv.kb.specialhi.x10.f = 0.0;
        st::ftAnim_8006EBA4(gobj);
    }
}

/// Air Shadow Ball entry.
pub fn ftKb_SpecialNMt_80107638(gobj: G) {
    melee_core::ct!("ftKb_SpecialNMt_80107638");
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        change_ms(gobj, ftKb_MS_MtSpecialAirNStart as i32, 0);
        st::Fighter_ClearCmdVars(&mut *fp);
        set_cbs(gobj);
        (*fp).mv.kb.specialhi.x0 = 0;
        (*fp).mv.kb.specialhi.x4 = 0;
        (*fp).mv.kb.specialhi.xC = if (*fp).u.kb.x9C == 0 { da.specialn_mt_frames_to_transition } else { 0 };
        (*fp).mv.kb.specialhi.x10.f = 0.0;
        (*fp).self_vel.y *= 0.5;
        st::ftAnim_8006EBA4(gobj);
    }
}

/// Spawn the held shadow ball once the cmd says so (cmd_vars[3] == 1, no ball yet).
/// `ftKb_SpecialNMt_CreateHeldShadow` (static inline); `pos1` / `pos2` are the C's scratch vectors.
pub fn ftKb_SpecialNMt_CreateHeldShadow(gobj: G, pos1: &mut Vec3, pos2: &mut Vec3) {
    melee_core::ct!("ftKb_SpecialNMt_CreateHeldShadow");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[3] == 1 && (*fp).u.kb.x98.is_null() {
            pos1.z = 2.0;
            pos1.y = 0.0;
            pos1.x = 0.0;
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_RHandN as usize)).joint as *mut _, pos1, pos2);
            pos2.z = 0.0;
            let shadow = ext::it_802C5000(gobj, pos2, FtPart_RHandN as i32, It_Kind_Kirby_MewtwoShadowBall as i32, (*fp).facing_dir);
            (*fp).u.kb.x98 = shadow;
            if !shadow.is_null() {
                set_cbs(gobj);
            } else {
                (*fp).u.kb.x98 = core::ptr::null_mut();
            }
        }
    }
}

unsafe fn start_anim(gobj: G, end_ms: i32, loop_ms: i32, clear_x0: bool) {
    let fp = fpp(gobj);
    let da = &*da_of(fp);
    let (mut sp2c, mut sp20) = (Vec3 { x: 0.0, y: 0.0, z: 0.0 }, Vec3 { x: 0.0, y: 0.0, z: 0.0 });
    ftKb_SpecialNMt_CreateHeldShadow(gobj, &mut sp2c, &mut sp20);
    if clear_x0 {
        (*fp).mv.kb.specialhi.x0 = 0;
    }
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        if (*fp).mv.kb.specialhi.x0 == 1 || (*fp).u.kb.x9C as f32 == da.specialn_mt_charge_time {
            change_ms(gobj, end_ms, 0);
        } else {
            change_ms(gobj, loop_ms, 0);
            (*fp).mv.kb.specialhi.x8.i = 0;
        }
        set_cbs(gobj);
    }
}

pub fn ftKb_MtSpecialNStart_Anim(gobj: G) {
    melee_core::ct!("ftKb_MtSpecialNStart_Anim");
    unsafe { start_anim(gobj, ftKb_MS_MtSpecialNEnd as i32, ftKb_MS_MtSpecialNLoop as i32, false) }
}

pub fn ftKb_MtSpecialAirNStart_Anim(gobj: G) {
    melee_core::ct!("ftKb_MtSpecialAirNStart_Anim");
    unsafe { start_anim(gobj, ftKb_MS_MtSpecialAirNEnd as i32, ftKb_MS_MtSpecialAirNLoop as i32, true) }
}

unsafe fn loop_anim(gobj: G, offset: &Vec3, full_ms: i32) {
    let fp = fpp(gobj);
    let da = &*da_of(fp);
    let sp34 = *offset;
    (*fp).mv.kb.specialhi.xC -= 1;
    if (*fp).mv.kb.specialhi.xC <= 0 {
        (*fp).mv.kb.specialhi.xC = 0;
        let (mut sp28, mut sp1c) = (Vec3 { x: 0.0, y: 0.0, z: 0.0 }, Vec3 { x: 0.0, y: 0.0, z: 0.0 });
        ftKb_SpecialNMt_CreateHeldShadow(gobj, &mut sp28, &mut sp1c);
        ftKb_SpecialNMt_80107410(gobj);
        if (*fp).mv.kb.specialhi.x8.i == 0 {
            (*fp).mv.kb.specialhi.x4 += 1;
            if (*fp).mv.kb.specialhi.x4 > da.specialn_mt_loops_to_full_charge as i32 {
                (*fp).mv.kb.specialhi.x4 = 0;
                (*fp).u.kb.x9C += 1;
                if (*fp).u.kb.x9C as f32 >= da.specialn_mt_charge_time {
                    st::Fighter_ChangeMotionState(gobj, full_ms, MF_LOOPFULL, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
                    (*fp).u.kb.x9C = da.specialn_mt_charge_time as i32;
                    (*fp).mv.kb.specialhi.x8.i = 1;
                    st::ftCommon_8007EBAC(&mut *fp, 0xC, 0);
                    let mut sp40 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                    ext::lb_8000B1CC((*(*fp).parts.add(FtPart_TopN as usize)).joint as *mut _, &sp34, &mut sp40);
                    ext::efSync_Spawn(0x1B, gobj, &[&mut sp40 as *mut Vec3 as *mut core::ffi::c_void]);
                    ext::ftCo_800BFFD0(fp, 0x5D, 0);
                }
            }
        }
    }
}

pub fn ftKb_MtSpecialNLoop_Anim(gobj: G) {
    melee_core::ct!("ftKb_MtSpecialNLoop_Anim");
    unsafe { loop_anim(gobj, &ftKb_Init_803B7570, ftKb_MS_MtSpecialNLoopFull as i32) }
}

pub fn ftKb_MtSpecialAirNLoop_Anim(gobj: G) {
    melee_core::ct!("ftKb_MtSpecialAirNLoop_Anim");
    unsafe { loop_anim(gobj, &ftKb_Init_803B757C, ftKb_MS_MtSpecialAirNLoopFull as i32) }
}

pub fn ftKb_MtSpecialNLoopFull_Anim(gobj: G) {
    melee_core::ct!("ftKb_MtSpecialNLoopFull_Anim");
    unsafe {
        let ft = fpp(gobj);
        let da = &*da_of(ft);
        (*ft).mv.kb.specialhi.x8.i = 1;
        (*ft).u.kb.x9C = da.specialn_mt_charge_time as i32;
    }
}

/// Note: the Air variant clears `x8.i` (the C differs from the ground one here).
pub fn ftKb_MtSpecialAirNLoopFull_Anim(gobj: G) {
    melee_core::ct!("ftKb_MtSpecialAirNLoopFull_Anim");
    unsafe {
        let ft = fpp(gobj);
        let da = &*da_of(ft);
        (*ft).mv.kb.specialhi.x8.i = 0;
        (*ft).u.kb.x9C = da.specialn_mt_charge_time as i32;
    }
}

pub fn ftKb_MtSpecialNCancel_Anim(gobj: G) {
    melee_core::ct!("ftKb_MtSpecialNCancel_Anim");
    unsafe {
        release_ball(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftKb_MtSpecialNEnd_Anim(gobj: G) {
    melee_core::ct!("ftKb_MtSpecialNEnd_Anim");
    unsafe {
        ftKb_SpecialNMt_801071FC(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

/// `ftKb_MtSpecialAirNCancel_Anim_inline` (static inline): identity.
pub fn ftKb_MtSpecialAirNCancel_Anim_inline(arg0: *mut Item_GObj) -> *mut Item_GObj {
    melee_core::ct!("ftKb_MtSpecialAirNCancel_Anim_inline");
    arg0
}

pub fn ftKb_MtSpecialAirNCancel_Anim(gobj: G) {
    melee_core::ct!("ftKb_MtSpecialAirNCancel_Anim");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            let new_var = !ftKb_MtSpecialAirNCancel_Anim_inline((*fp).u.kb.x98).is_null();
            if new_var {
                ext::it_802C573C((*fp).u.kb.x98);
                (*fp).u.kb.x98 = core::ptr::null_mut();
            }
            if !gobj.is_null() {
                let fp2 = fpp(gobj);
                st::efLib_DestroyAll(gobj);
                (*fp2).u.kb.xA0 = core::ptr::null_mut();
            }
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

pub fn ftKb_MtSpecialAirNEnd_Anim(gobj: G) {
    melee_core::ct!("ftKb_MtSpecialAirNEnd_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        ftKb_SpecialNMt_801071FC(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if da.specialn_mt_freefall_toggle == 0.0 {
                st::ftCo_Fall_Enter(gobj);
                return;
            }
            st::ftCo_80096900(gobj, 1, 0, 1, 1.0, da.specialn_mt_freefall_toggle);
        }
    }
}

pub fn ftKb_MtSpecialNStart_IASA(_gobj: G) { melee_core::ct!("ftKb_MtSpecialNStart_IASA");}
pub fn ftKb_MtSpecialNCancel_IASA(_gobj: G) { melee_core::ct!("ftKb_MtSpecialNCancel_IASA");}
pub fn ftKb_MtSpecialNEnd_IASA(_gobj: G) { melee_core::ct!("ftKb_MtSpecialNEnd_IASA");}
pub fn ftKb_MtSpecialAirNStart_IASA(_gobj: G) { melee_core::ct!("ftKb_MtSpecialAirNStart_IASA");}
pub fn ftKb_MtSpecialAirNCancel_IASA(_gobj: G) { melee_core::ct!("ftKb_MtSpecialAirNCancel_IASA");}
pub fn ftKb_MtSpecialAirNEnd_IASA(_gobj: G) { melee_core::ct!("ftKb_MtSpecialAirNEnd_IASA");}

const SIGN_BIT: u32 = 0x8000_0000;

/// Loop IASA (ground): A or B leaves for End once the transition timer is done, the sign bit
/// (the C's `0x80000000`) cancels; a caught-state check aborts first.
pub fn ftKb_MtSpecialNLoop_IASA(gobj: G) {
    melee_core::ct!("ftKb_MtSpecialNLoop_IASA");
    unsafe {
        let fp = fpp(gobj);
        if ext::ftCo_8009917C(gobj) {
            release_ball(gobj);
        } else {
            let input = (*fp).input.pressed_buttons;
            if (input & HSD_PAD_A as u32) != 0 && (*fp).mv.kb.specialhi.xC <= 0 {
                change_ms(gobj, ftKb_MS_MtSpecialNEnd as i32, 0);
                set_cbs(gobj);
                return;
            }
            if (input & HSD_PAD_B as u32) != 0 {
                if (*fp).mv.kb.specialhi.xC <= 0 {
                    change_ms(gobj, ftKb_MS_MtSpecialNEnd as i32, 0);
                    set_cbs(gobj);
                }
            } else if (input & SIGN_BIT) != 0 {
                change_ms(gobj, ftKb_MS_MtSpecialNCancel as i32, 0);
                release_ball(gobj);
                set_cbs(gobj);
            }
        }
    }
}

pub fn ftKb_MtSpecialNLoopFull_IASA(gobj: G) {
    melee_core::ct!("ftKb_MtSpecialNLoopFull_IASA");
    unsafe {
        let fp = fpp(gobj);
        if ext::ftCo_8009917C(gobj) {
            release_ball(gobj);
        } else {
            let input = (*fp).input.pressed_buttons;
            if (input & HSD_PAD_A as u32) != 0 || (input & HSD_PAD_B as u32) != 0 {
                change_ms(gobj, ftKb_MS_MtSpecialNEnd as i32, 0);
                set_cbs(gobj);
                return;
            }
            if (input & SIGN_BIT) != 0 {
                change_ms(gobj, ftKb_MS_MtSpecialNCancel as i32, 0);
                release_ball(gobj);
                set_cbs(gobj);
            }
        }
    }
}

pub fn ftKb_MtSpecialAirNLoop_IASA(gobj: G) {
    melee_core::ct!("ftKb_MtSpecialAirNLoop_IASA");
    unsafe {
        let fp = fpp(gobj);
        let input = (*fp).input.pressed_buttons;
        if (input & HSD_PAD_A as u32) != 0 && (*fp).mv.kb.specialhi.xC <= 0 {
            change_ms(gobj, ftKb_MS_MtSpecialAirNEnd as i32, 0);
            set_cbs(gobj);
            return;
        }
        if (input & HSD_PAD_B as u32) != 0 {
            if (*fp).mv.kb.specialhi.xC <= 0 {
                change_ms(gobj, ftKb_MS_MtSpecialAirNEnd as i32, 0);
                set_cbs(gobj);
            }
        } else if (input & SIGN_BIT) != 0 {
            change_ms(gobj, ftKb_MS_MtSpecialAirNCancel as i32, 0);
            release_ball(gobj);
            set_cbs(gobj);
        }
    }
}

pub fn ftKb_MtSpecialAirNLoopFull_IASA(gobj: G) {
    melee_core::ct!("ftKb_MtSpecialAirNLoopFull_IASA");
    unsafe {
        let input = (*fpp(gobj)).input.pressed_buttons;
        if (input & HSD_PAD_A as u32) != 0 || (input & HSD_PAD_B as u32) != 0 {
            change_ms(gobj, ftKb_MS_MtSpecialAirNEnd as i32, 0);
            set_cbs(gobj);
            return;
        }
        if (input & SIGN_BIT) != 0 {
            change_ms(gobj, ftKb_MS_MtSpecialAirNCancel as i32, 0);
            release_ball(gobj);
            set_cbs(gobj);
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftKb_MtSpecialNStart_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialNStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialNStart_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80082708(gobj) == 0 /* GA_Ground */) {
            st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 505 /* ftKb_MS_MtSpecialAirNStart */, 0xC4C5080u32 /* ftCommon_GroundAirColl_MF */);
            (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
        }
    }
}

/// `ftKb_MtSpecialNLoop_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialNLoop_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialNLoop_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80082708(gobj) == 0 /* GA_Ground */) {
            st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 506 /* ftKb_MS_MtSpecialAirNLoop */, 0xC4C5280u32 /* ftMt_MF_SpecialNLoop_Coll */);
            (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
        }
    }
}

/// `ftKb_MtSpecialNLoopFull_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialNLoopFull_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialNLoopFull_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80082708(gobj) == 0 /* GA_Ground */) {
            st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 507 /* ftKb_MS_MtSpecialAirNLoopFull */, 0xC4C5280u32 /* ftMt_MF_SpecialNLoop_Coll */);
            (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
        }
    }
}

/// `ftKb_MtSpecialNCancel_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialNCancel_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialNCancel_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80082708(gobj) == 0 /* GA_Ground */) {
            st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 508 /* ftKb_MS_MtSpecialAirNCancel */, 0xC4C5080u32 /* ftCommon_GroundAirColl_MF */);
            (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
        }
    }
}

/// `ftKb_MtSpecialNEnd_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialNEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialNEnd_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80082708(gobj) == 0 /* GA_Ground */) {
            st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 509 /* ftKb_MS_MtSpecialAirNEnd */, 0xC4C5080u32 /* ftCommon_GroundAirColl_MF */);
            (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
        }
    }
}

/// `ftKb_MtSpecialAirNStart_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialAirNStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialAirNStart_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80081D0C(gobj) == 1 /* GA_Air */) {
            st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 500 /* ftKb_MS_MtSpecialNStart */, 0xC4C5080u32 /* ftCommon_GroundAirColl_MF */);
            (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
        }
    }
}

/// `ftKb_MtSpecialAirNLoop_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialAirNLoop_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialAirNLoop_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80081D0C(gobj) == 1 /* GA_Air */) {
            st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 501 /* ftKb_MS_MtSpecialNLoop */, 0xC4C5280u32 /* ftMt_MF_SpecialNLoop_Coll */);
            (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
        }
    }
}

/// `ftKb_MtSpecialAirNLoopFull_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialAirNLoopFull_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialAirNLoopFull_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80081D0C(gobj) == 1 /* GA_Air */) {
            st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 502 /* ftKb_MS_MtSpecialNLoopFull */, 0xC4C5280u32 /* ftMt_MF_SpecialNLoop_Coll */);
            (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
        }
    }
}

/// `ftKb_MtSpecialAirNCancel_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialAirNCancel_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialAirNCancel_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80081D0C(gobj) == 1 /* GA_Air */) {
            st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 503 /* ftKb_MS_MtSpecialNCancel */, 0xC4C5080u32 /* ftCommon_GroundAirColl_MF */);
            (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
        }
    }
}

/// `ftKb_MtSpecialAirNEnd_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialAirNEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialAirNEnd_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80081D0C(gobj) == 1 /* GA_Air */) {
            st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 504 /* ftKb_MS_MtSpecialNEnd */, 0xC4C5080u32 /* ftCommon_GroundAirColl_MF */);
            (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
        }
    }
}

/// `ftKb_MtSpecialNStart_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialNStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialNStart_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_MtSpecialNLoop_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialNLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialNLoop_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_MtSpecialNLoopFull_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialNLoopFull_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialNLoopFull_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_MtSpecialNCancel_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialNCancel_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialNCancel_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_MtSpecialNEnd_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialNEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialNEnd_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_MtSpecialAirNStart_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialAirNStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialAirNStart_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftKb_MtSpecialAirNLoop_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialAirNLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialAirNLoop_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftKb_MtSpecialAirNLoopFull_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialAirNLoopFull_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialAirNLoopFull_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftKb_MtSpecialAirNCancel_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialAirNCancel_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialAirNCancel_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftKb_MtSpecialAirNEnd_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c).
pub fn ftKb_MtSpecialAirNEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MtSpecialAirNEnd_Phys");
    st::ft_80084EEC(gobj);
}
