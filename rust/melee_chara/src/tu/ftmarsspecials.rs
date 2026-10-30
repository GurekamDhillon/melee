//! src/melee/ft/kinds/ftMars/ftmarsspecials.c: Dancing Blade (side B), the four-stage combo.
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function). Smoke-tested by
//! tests/tu_ftmarsspecials.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_fighter::ftcommon as fc;
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (rounds 2, 3, 4, 13): re-exported under this TU.
pub use crate::chars::r13::ftMars::ftmarsspecials::{ftMs_SpecialS3_Phys, ftMs_SpecialS4_Phys};
pub use crate::chars::r2::ftMars::ftmarsspecials::ftMs_SpecialS_801376E8;
pub use crate::chars::r3::ftMars::ftmarsspecials::ftMs_SpecialS_80137748;
pub use crate::chars::r4::ftMars::ftmarsspecials::{
    ftMs_SpecialAirS1_Anim, ftMs_SpecialAirS1_Coll, ftMs_SpecialS2_Anim, ftMs_SpecialS3_Anim, ftMs_SpecialS4_Anim,
};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    pub static mut FT_800892A0: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ft_800892A0(g: *mut HSD_GObj) {
        match unsafe { FT_800892A0 } {
            Some(f) => f(g),
            None => crate::adapt::ft_800892A0(g),
        }
    }
}

/// `HSD_PAD_A | HSD_PAD_B`.
const HSD_PAD_AB: u32 = (HSD_PAD_A | HSD_PAD_B) as u32;

/// Ft_MF_KeepColAnimHitStatus | SkipHit | SkipMatAnim | SkipColAnim | UpdateCmd | KeepSwordTrail |
/// SkipItemVis | Unk19 | SkipModelPartVis | SkipModelFlags | Unk27.
const transition_flags: u32 = (Ft_MF_KeepColAnimHitStatus
    | Ft_MF_SkipHit
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_KeepSwordTrail
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *const MarsAttributes {
    (*fp).dat_attrs as *const MarsAttributes
}

/// `ftMs_SpecialS_80137A68` as a Fighter callback entry (`x21EC`).
pub unsafe extern "C" fn cb_ftMs_SpecialS_80137A68(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftMs_SpecialS_80137A68");
    ftMs_SpecialS_80137A68(g);
}

/// Common tail of both Enter functions: pick the ground/air first-hit state and start it.
unsafe fn start_s1(gobj: *mut HSD_GObj, fp: *mut Fighter) {
    (*fp).cmd_vars[1] = 0;
    (*fp).cmd_vars[0] = 0;
    (*fp).mv.ms.specials.x0 = 0;
    let msid: i32 = if (*fp).ground_or_air == GA_Ground { 349 } else { 358 };
    st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
}

pub fn ftMs_SpecialS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialS_Enter");
    unsafe {
        let fp = fpp(gobj);
        (*fp).self_vel.y = 0.0;
        start_s1(gobj, fp);
    }
}

pub fn ftMs_SpecialAirS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirS_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).self_vel.x = (*fp).self_vel.x / (*da).x14;
        if (*fp).u.ms.x222C as i32 == 0 {
            (*fp).u.ms.x222C = 1;
            (*fp).self_vel.y = (*da).x1C;
        } else {
            (*fp).self_vel.y = 0.0;
        }
        start_s1(gobj, fp);
    }
}

/// Shared IASA body: buffer a second press before the window opens, chain once it is open.
unsafe fn chain_iasa(gobj: *mut HSD_GObj, next: fn(*mut HSD_GObj)) {
    let fp = fpp(gobj);
    if (*fp).cmd_vars[0] != 0 {
        if (*fp).cmd_vars[1] == 0 && ((*fp).input.pressed_buttons & HSD_PAD_AB) != 0 {
            next(gobj);
        }
    } else if ((*fp).input.pressed_buttons & HSD_PAD_AB) != 0 {
        (*fp).cmd_vars[1] = 1;
    }
}

pub fn ftMs_SpecialAirS1_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirS1_IASA");
    unsafe { chain_iasa(gobj, ftMs_SpecialS_80137A9C) }
}

/// Phys for stages 1 and 2 (identical in the C).
unsafe fn phys12(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let da = da(fp);
    if (*fp).ground_or_air == GA_Ground {
        st::ft_80084F3C(gobj);
    } else {
        fc::fall(&mut *fp, (*da).x20, (*da).x24);
        fc::calc_self_accel_deaccel(&mut *fp, (*da).x18);
    }
}

pub fn ftMs_SpecialAirS1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirS1_Phys");
    unsafe { phys12(gobj) }
}

pub fn ftMs_SpecialS2_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialS2_IASA");
    unsafe { chain_iasa(gobj, ftMs_SpecialS_80137E0C) }
}

pub fn ftMs_SpecialS2_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialS2_Phys");
    unsafe { phys12(gobj) }
}

pub fn ftMs_SpecialS2_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialS2_Coll");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Ground {
            if st::ft_800827A0(gobj) == 0 {
                ftMs_SpecialS_80137940(gobj);
            }
        } else if st::ft_80081D0C(gobj) != 0 {
            ftMs_SpecialS_801379D0(gobj);
        }
    }
}

/// `switch (fp->motion_id)` with no default: the C leaves `msid` uninitialised for other ids
/// (gwtool zeroes locals), so 0 here.
fn pick(motion_id: i32, table: &[(i32, i32)]) -> i32 {
    for &(from, to) in table {
        if motion_id == from {
            return to;
        }
    }
    0
}

pub fn ftMs_SpecialS_80137940(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialS_80137940");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D5D4(&mut *fp);
        let msid = pick((*fp).motion_id as i32, &[(350, 359), (351, 360)]);
        st::Fighter_ChangeMotionState(gobj, msid, transition_flags, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    }
}

pub fn ftMs_SpecialS_801379D0(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialS_801379D0");
    unsafe {
        let fp = fpp(gobj);
        (*fp).u.ms.x222C = 0;
        st::ftCommon_8007D7FC(&mut *fp);
        let msid = pick((*fp).motion_id as i32, &[(359, 350), (360, 351)]);
        st::Fighter_ChangeMotionState(gobj, msid, transition_flags, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    }
}

/// Callback installed in `x21EC` by the chain starters: reset the attack counter.
pub fn ftMs_SpecialS_80137A68(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialS_80137A68");
    melee_fighter::ft0892::ft_80089824(gobj);
    ext::ft_800892A0(gobj);
}

/// Start the next stage; `ids` = (ground, air) for each stick region.
unsafe fn chain_start(gobj: *mut HSD_GObj, pick_ids: &dyn Fn(f32) -> (i32, i32)) {
    let fp = fpp(gobj);
    (*fp).cmd_vars[1] = 0;
    (*fp).cmd_vars[0] = 0;
    (*fp).x21EC = Some(cb_ftMs_SpecialS_80137A68);
    let (ground, air) = pick_ids((*fp).input.lstick[0].y);
    let msid = if (*fp).ground_or_air == GA_Ground { ground } else { air };
    st::Fighter_ChangeMotionState(gobj, msid, Ft_MF_SkipAttackCount as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
}

pub fn ftMs_SpecialS_80137A9C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialS_80137A9C");
    unsafe {
        let x21c = fh::common_data().x21C;
        chain_start(gobj, &|y| if y > x21c { (350, 359) } else { (351, 360) });
    }
}

pub fn ftMs_SpecialS3_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialS3_IASA");
    unsafe { chain_iasa(gobj, ftMs_SpecialS_80138148) }
}

pub fn ftMs_SpecialS3_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialS3_Coll");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Ground {
            if st::ft_800827A0(gobj) == 0 {
                ftMs_SpecialS_80137CBC(gobj);
            }
        } else if st::ft_80081D0C(gobj) != 0 {
            ftMs_SpecialS_80137D60(gobj);
        }
    }
}

pub fn ftMs_SpecialS_80137CBC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialS_80137CBC");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D5D4(&mut *fp);
        let msid = pick((*fp).motion_id as i32, &[(352, 361), (354, 363), (353, 362)]);
        st::Fighter_ChangeMotionState(gobj, msid, transition_flags, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    }
}

pub fn ftMs_SpecialS_80137D60(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialS_80137D60");
    unsafe {
        let fp = fpp(gobj);
        (*fp).u.ms.x222C = 0;
        st::ftCommon_8007D7FC(&mut *fp);
        let msid = pick((*fp).motion_id as i32, &[(361, 352), (363, 354), (362, 353)]);
        st::Fighter_ChangeMotionState(gobj, msid, transition_flags, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    }
}

pub fn ftMs_SpecialS_80137E0C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialS_80137E0C");
    unsafe {
        let x21c = fh::common_data().x21C;
        chain_start(gobj, &|y| {
            if y > x21c {
                (352, 361)
            } else if y < -x21c {
                (354, 363)
            } else {
                (353, 362)
            }
        });
    }
}

pub fn ftMs_SpecialS4_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMs_SpecialS4_IASA");}

pub fn ftMs_SpecialS4_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialS4_Coll");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Ground {
            if st::ft_800827A0(gobj) == 0 {
                ftMs_SpecialS_80137FF8(gobj);
            }
        } else if st::ft_80081D0C(gobj) != 0 {
            ftMs_SpecialS_8013809C(gobj);
        }
    }
}

pub fn ftMs_SpecialS_80137FF8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialS_80137FF8");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D5D4(&mut *fp);
        let msid = pick((*fp).motion_id as i32, &[(355, 364), (357, 366), (356, 365)]);
        st::Fighter_ChangeMotionState(gobj, msid, transition_flags, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    }
}

pub fn ftMs_SpecialS_8013809C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialS_8013809C");
    unsafe {
        let fp = fpp(gobj);
        (*fp).u.ms.x222C = 0;
        st::ftCommon_8007D7FC(&mut *fp);
        let msid = pick((*fp).motion_id as i32, &[(364, 355), (366, 357), (365, 356)]);
        st::Fighter_ChangeMotionState(gobj, msid, transition_flags, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    }
}

pub fn ftMs_SpecialS_80138148(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialS_80138148");
    unsafe {
        let x21c = fh::common_data().x21C;
        chain_start(gobj, &|y| {
            if y > x21c {
                (355, 364)
            } else if y < -x21c {
                (357, 366)
            } else {
                (356, 365)
            }
        });
    }
}
