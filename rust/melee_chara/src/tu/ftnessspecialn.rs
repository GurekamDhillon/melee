//! src/melee/ft/kinds/ftNess/ftnessspecialn.c: Ness neutral B (PK Flash).
//!
//! One fused site, `vec.y += 3.0f * scale.y` in `ftNs_PKFlash_Init` (tools/rust-port/fma_sites.py),
//! written with [`fmadds`]. Smoke-tested through recording hooks by tests/tu_ftnessspecialn.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::chars::caps_max_jumps;
use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2/4/6, chars/): re-exported under this TU (their signatures take the
/// gobj or the fighter exactly as the earlier round decided).
pub use crate::chars::ftNess::ftnessspecialn::{
    ftNs_SpecialAirNEnd_Phys, ftNs_SpecialAirNRelease_Phys, ftNs_SpecialAirNStart_Phys,
};
pub use crate::chars::r2::ftNess::ftnessspecialn::{
    ftNs_SpecialAirNEnd_Coll, ftNs_SpecialAirNRelease_Coll, ftNs_SpecialAirNStart_Coll,
    ftNs_SpecialNEnd_Coll, ftNs_SpecialNEnd_Phys, ftNs_SpecialNRelease_Coll,
    ftNs_SpecialNRelease_Phys, ftNs_SpecialNStart_Coll,
};
pub use crate::chars::r4::ftNess::ftnessspecialn::SetPKFlashAttr;
pub use crate::chars::r6::ftNess::ftnessspecialn::{ftNs_SpecialNStart_Phys, GravityDelay};

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
    pub static mut IT_802AAA50: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802AAA50(item: *mut HSD_GObj)  {
                match unsafe { IT_802AAA50 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itnesspkflash::it_802AAA50(item as _); } return; },
                }
            }
    pub static mut IT_802AA8C0: Option<fn(*mut HSD_GObj, *mut Vec3, i32, f32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802AA8C0(owner: *mut HSD_GObj, pos: *mut Vec3, kind: i32, facing: f32) -> *mut HSD_GObj {
                match unsafe { IT_802AA8C0 } {
                    Some(f) => f(owner, pos, kind, facing),
                    None => { return unsafe { melee_items::tu::itnesspkflash::it_802AA8C0(owner as _, pos as _, kind as _, facing as _) as _ }; },
                }
            }
    pub static mut IT_802AA7E4: Option<fn(*mut HSD_GObj) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802AA7E4(item: *mut HSD_GObj) -> *mut HSD_GObj {
                match unsafe { IT_802AA7E4 } {
                    Some(f) => f(item),
                    None => { return unsafe { melee_items::tu::itnesspkflash::it_802AA7E4(item as _) as _ }; },
                }
            }
    pub static mut IT_802AA7F0: Option<fn(*mut HSD_GObj) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn it_802AA7F0(item: *mut HSD_GObj) -> i32 {
                match unsafe { IT_802AA7F0 } {
                    Some(f) => f(item),
                    None => { return unsafe { melee_items::tu::itnesspkflash::it_802AA7F0(item as _) as _ }; },
                }
            }
    pub static mut FTPARTSETROTX: Option<fn(*mut Fighter, i32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn ftPartSetRotX(fp: *mut Fighter, part: i32, v: f32) {
        match unsafe { FTPARTSETROTX } {
            Some(f) => f(fp, part, v),
            None => { unsafe { let _ = melee_fighter::tu::ftparts::ftPartSetRotX(&*(fp as *const _), part as _, v as _); } },
        }
    }
}

/// `FTNESS_SPECIALN_COLL_FLAG`.
pub const FTNESS_SPECIALN_COLL_FLAG: u32 = Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// C `bool`: Ness is charging a live PK Flash (grounded or aerial Hold motion).
pub fn ftNs_SpecialN_CheckSpecialNHold(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftNs_SpecialN_CheckSpecialNHold");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() && !(*fp).u.ns.pkflash_gobj.is_null() {
                let msid = (*fp).motion_id as i32;
                if msid == ftNs_MS_SpecialNHold as i32 || msid == ftNs_MS_SpecialAirNHold as i32 {
                    return true;
                }
            }
        }
        false
    }
}

/// Clear Ness's callbacks and PK Flash's item GObj pointer.
pub fn ftNs_SpecialN_SetNULL(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialN_SetNULL");
    unsafe {
        if gobj.is_null() {
            return;
        }
        let fp = fpp(gobj);
        if fp.is_null() {
            return;
        }
        if !(*fp).u.ns.pkflash_gobj.is_null() {
            (*fp).u.ns.pkflash_gobj = core::ptr::null_mut();
        }
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
    }
}

/// Clear PK Flash's owner pointer, Ness's pointer to the item, and reset callbacks.
pub fn ftNs_SpecialN_ItemPKFlushSetNULL(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialN_ItemPKFlushSetNULL");
    unsafe {
        if gobj.is_null() {
            return;
        }
        let fp = fpp(gobj);
        if fp.is_null() {
            return;
        }
        if !(*fp).u.ns.pkflash_gobj.is_null() {
            ext::it_802AAA50((*fp).u.ns.pkflash_gobj);
            (*fp).u.ns.pkflash_gobj = core::ptr::null_mut();
        }
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
    }
}

pub fn ftNs_PKFlash_Init(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_PKFlash_Init");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).u.ns.pkflash_gobj.is_null() {
            let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let joint = (*(*fp).parts.add(FtPart_L2ndNa as usize)).joint as *mut HSD_JObj;
            ext::lb_8000B1CC(joint, core::ptr::null(), &mut vec);
            vec.z = 0.0;
            vec.y = fmadds(3.0, (*fp).x34_scale.y, vec.y);

            let flash = ext::it_802AA8C0(gobj, &mut vec, It_Kind_Ness_PKFlush as i32, (*fp).facing_dir);
            (*fp).u.ns.pkflash_gobj = flash;
            if !flash.is_null() {
                (*fp).death2_cb = Some(crate::tu::ftnessattackhi4::cb_ftNs_Init_OnDamage);
                (*fp).take_dmg_cb = Some(crate::tu::ftnessattackhi4::cb_ftNs_Init_OnDamage);
            }
        }
    }
}

/// The attribute reset shared by both Start enters (and, via `SetPKFlashAttr`, both End anims).
unsafe fn reset_pkflash_attrs(fp: *mut Fighter) {
    let sa = (*fp).dat_attrs as *const ftNessAttributes;
    (*fp).mv.ns.specialn.frames_to_loop_charge_ground = (*sa).x0_PKFLASH_TIMER1_LOOPFRAMES;
    (*fp).mv.ns.specialn.frames_to_loop_charge_air = (*sa).x4_PKFLASH_TIMER2_LOOPFRAMES;
    (*fp).mv.ns.specialn.falling_acceleration_delay = (*sa).x8_PKFLASH_GRAVITY_DELAY;
    (*fp).u.ns.pkflash_gobj = core::ptr::null_mut();
    (*fp).mv.ns.specialn.charge_release_delay = (*sa).xC_PKFLASH_MINCHARGEFRAMES;
    (*fp).death2_cb = None;
    (*fp).take_dmg_cb = None;
}

pub fn ftNs_SpecialNStart_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialNStart_Enter");
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(gobj, ftNs_MS_SpecialNStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::Fighter_ClearCmdVars(&mut *fp);
        reset_pkflash_attrs(fp);
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftNs_SpecialAirNStart_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirNStart_Enter");
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(gobj, ftNs_MS_SpecialAirNStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::Fighter_ClearCmdVars(&mut *fp);
        (*fp).self_vel.y = 0.0;
        reset_pkflash_attrs(fp);
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftNs_SpecialNStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialNStart_Anim");
    unsafe {
        let fp = fpp(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) != 0 {
            return;
        }
        st::Fighter_ChangeMotionState(gobj, ftNs_MS_SpecialNHold as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ftNs_PKFlash_Init(gobj);
        (*fp).x1968_jumpsUsed = caps_max_jumps(&*fp) as u8;
    }
}

/// Shared charge animation of the grounded/aerial Hold motions: count the loop timers down, end
/// the move when the flash is gone and both timers ran out, or switch to the Release motion.
unsafe fn release_anim(gobj: *mut HSD_GObj, end_msid: i32, release_msid: i32) {
    let fp = fpp(gobj);
    if (*fp).mv.ns.specialn.frames_to_loop_charge_ground != 0 {
        (*fp).mv.ns.specialn.frames_to_loop_charge_ground -= 1;
    }
    if (*fp).u.ns.pkflash_gobj.is_null() && (*fp).mv.ns.specialn.frames_to_loop_charge_air != 0 {
        (*fp).mv.ns.specialn.frames_to_loop_charge_air -= 1;
    }

    if (*fp).u.ns.pkflash_gobj.is_null() {
        if (*fp).mv.ns.specialn.frames_to_loop_charge_ground <= 0
            && (*fp).mv.ns.specialn.frames_to_loop_charge_air <= 0
        {
            st::Fighter_ChangeMotionState(gobj, end_msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            return;
        }
        if (*fp).motion_id as i32 != release_msid {
            st::Fighter_ChangeMotionState(gobj, release_msid, 0, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        }
        return;
    }

    if ext::it_802AA7E4((*fp).u.ns.pkflash_gobj) != gobj {
        (*fp).u.ns.pkflash_gobj = core::ptr::null_mut();
        return;
    }
    if ext::it_802AA7F0((*fp).u.ns.pkflash_gobj) == 1 && (*fp).motion_id as i32 != release_msid {
        st::Fighter_ChangeMotionState(gobj, release_msid, 0, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    }
}

pub fn ftNs_SpecialNRelease_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialNRelease_Anim");
    unsafe { release_anim(gobj, ftNs_MS_SpecialNEnd as i32, ftNs_MS_SpecialNRelease as i32) }
}

pub fn ftNs_SpecialNEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialNEnd_Anim");
    unsafe {
        let fp = fpp(gobj);
        SetPKFlashAttr(gobj);
        ext::ftPartSetRotX(fp, 0, 0.0);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftNs_SpecialAirNStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirNStart_Anim");
    unsafe {
        let fp = fpp(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) != 0 {
            return;
        }
        st::Fighter_ChangeMotionState(gobj, ftNs_MS_SpecialAirNHold as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ftNs_PKFlash_Init(gobj);
        (*fp).x1968_jumpsUsed = caps_max_jumps(&*fp) as u8;
    }
}

pub fn ftNs_SpecialAirNRelease_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirNRelease_Anim");
    unsafe { release_anim(gobj, ftNs_MS_SpecialAirNEnd as i32, ftNs_MS_SpecialAirNRelease as i32) }
}

pub fn ftNs_SpecialAirNEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirNEnd_Anim");
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftNessAttributes;
        SetPKFlashAttr(gobj);
        ext::ftPartSetRotX(fp, 0, 0.0);
        if st::ftAnim_IsFramesRemaining(gobj) != 0 {
            return;
        }
        let landing_lag = (*sa).x1C_PKFLASH_LANDING_LAG;
        if landing_lag == 0.0 {
            st::ftCo_Fall_Enter(gobj);
            return;
        }
        st::ftCo_80096900(gobj, 1, 0, 1, 1.0, (*sa).x1C_PKFLASH_LANDING_LAG);
    }
}

pub fn ftNs_SpecialNStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftNs_SpecialNStart_IASA");}

/// Shared IASA of the two Hold motions: after the minimum charge, letting go of B drops the flash.
unsafe fn release_iasa(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    (*fp).mv.ns.specialn.charge_release_delay -= 1;
    if (*fp).mv.ns.specialn.charge_release_delay <= 0 {
        (*fp).mv.ns.specialn.charge_release_delay = 0;
    } else {
        return;
    }
    if (*fp).input.held_buttons[0] as i32 & HSD_PAD_B != 0 {
        return;
    }
    ftNs_SpecialN_SetNULL(gobj);
}

pub fn ftNs_SpecialNRelease_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialNRelease_IASA");
    unsafe { release_iasa(gobj) }
}

pub fn ftNs_SpecialNEnd_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftNs_SpecialNEnd_IASA");}

pub fn ftNs_SpecialAirNStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftNs_SpecialAirNStart_IASA");}

pub fn ftNs_SpecialAirNRelease_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirNRelease_IASA");
    unsafe { release_iasa(gobj) }
}

pub fn ftNs_SpecialAirNEnd_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftNs_SpecialAirNEnd_IASA");}
