//! src/melee/ft/kinds/ftPopo/ftpopospecialhi.c: Ice Climbers' up-B (Belay) and the Squall Hammer
//! string helpers. One fused f32 site, in ftPp_SpecialS_80120E68 (`-(3.0 * f2.facing_dir - vel.x)`
//! is `-fmadds(3.0, facing_dir, -vel.x)`), and the three f64 Newton steps of `my_sqrtf`
//! (`crate::pure::my_sqrtf`); tools/rust-port/fma_sites.py. Smoke-tested by
//! tests/tu_ftpopospecialhi.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_fighter::ftcommon as co;
use melee_types::*;

/// Already ported (rounds 2/3/4/7/11 and hand): re-exported under this TU.
pub use crate::chars::r2::ftPopo::ftpopospecialhi::{
    ftPp_SpecialHi_80121CE0, ftPp_SpecialHi_80121D40, ftPp_SpecialHi_80121DA0, ftPp_SpecialHi_80121DD8,
    ftPp_SpecialHi_80121FD8, ftPp_SpecialHi_80122038, ftPp_SpecialHi_80122098, ftPp_SpecialHi_801220D4,
    ftPp_SpecialHi_801217EC, ftPp_SpecialHi_8012184C, ftPp_SpecialHi_801218AC, ftPp_SpecialHi_801218F8,
    ftPp_SpecialHi_801222E8, ftPp_SpecialHi_80122380, ftPp_SpecialHi_801227AC, ftPp_SpecialHiStart_1_Phys,
    ftPp_SpecialHiThrow_1_Anim, ftPp_SpecialHiThrow_1_Phys,
};
pub use crate::chars::r3::ftPopo::ftpopospecialhi::{
    ftPp_SpecialHiStart_0_Coll, ftPp_SpecialHiStart_1_Coll, ftPp_SpecialHiThrow2_Coll, ftPp_SpecialHiThrow_0_Coll,
    ftPp_SpecialHiThrow_1_Coll, ftPp_SpecialS_8012114C,
};
pub use crate::chars::r4::ftPopo::ftpopospecialhi::ftPp_SpecialAirHiStart_1_Anim;
pub use crate::chars::r7::ftPopo::ftpopospecialhi::ftPp_SpecialAirHiThrow_1_Anim;
pub use crate::chars::r11::ftPopo::ftpopospecialhi::{
    ftPp_SpecialAirHiStart_0_Coll, ftPp_SpecialAirHiStart_1_Coll, ftPp_SpecialAirHiThrow_0_Coll,
    ftPp_SpecialAirHiThrow_1_Coll,
};

/// Gobj-signature wrapper over the hand port in `chars::ftPopo` (which takes the fighter).
pub fn ftPp_SpecialAirHiThrow_1_Phys(gobj: G) {
    unsafe { crate::chars::ftPopo::ftpopospecialhi::ftPp_SpecialAirHiThrow_1_Phys(&mut *fpp(gobj)) }
}

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
    pub static mut PLAYER_GETENTITYATINDEX: Option<fn(i32, i32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn Player_GetEntityAtIndex(slot: i32, index: i32) -> *mut HSD_GObj {
                match unsafe { PLAYER_GETENTITYATINDEX } {
                    Some(f) => f(slot, index),
                    None => { return unsafe { melee_fighter::tu::player::Player_GetEntityAtIndex(slot as _, index as _) as _ }; },
                }
            }
    pub static mut FTNN_INIT_8012300C: Option<fn(*mut HSD_GObj) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn ftNn_Init_8012300C(gobj: *mut HSD_GObj) -> i32 {
                match unsafe { FTNN_INIT_8012300C } {
                    Some(f) => f(gobj),
                    None => { return unsafe { crate::tu::ftnana::ftNn_Init_8012300C(gobj as _) as _ }; },
                }
            }
    pub static mut FTNN_INIT_8012309C: Option<fn(*mut HSD_GObj) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn ftNn_Init_8012309C(gobj: *mut HSD_GObj) -> i32 {
                match unsafe { FTNN_INIT_8012309C } {
                    Some(f) => f(gobj),
                    None => { return unsafe { crate::tu::ftnana::ftNn_Init_8012309C(gobj as _) as _ }; },
                }
            }
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, off, out),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, off as _, out as _); } return; },
                }
            }
    hook!(IT_802C3950, it_802C3950, (item: *mut HSD_GObj));
    hook!(IT_802C3810, it_802C3810, (item: *mut HSD_GObj));
    hook!(IT_802C3864, it_802C3864, (item: *mut HSD_GObj));
    hook!(IT_802C2750, it_802C2750, (item: *mut HSD_GObj));
    hook!(IT_802C27D4, it_802C27D4, (parent: *mut HSD_GObj, pos: *mut Vec3, motion_id: u32, dir: f32) -> *mut HSD_GObj);
    /// Recording hook; unset, the native `melee_fighter::tu::ftparts` port runs.
    pub static mut FTPARTSETROTY: Option<fn(*mut Fighter, i32, f32)> = None;
    pub fn ftPartSetRotY(fp: *mut Fighter, part: i32, v: f32) {
        match unsafe { FTPARTSETROTY } {
            Some(f) => f(fp, part, v),
            None => melee_fighter::tu::ftparts::ftPartSetRotY(unsafe { &*fp }, part, v),
        }
    }
}

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpp(gobj: G) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *const ftIceClimberAttributes {
    (*fp).dat_attrs as *const ftIceClimberAttributes
}
/// C `ABS` on floats: `x < 0 ? -x : x` (so -0.0 stays -0.0).
#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

/// `ftGetFacingDirInt` (ft/inlines.h).
#[inline(always)]
unsafe fn ftGetFacingDirInt(fp: *mut Fighter) -> i32 {
    if (*fp).facing_dir < 0.0 {
        -1
    } else {
        1
    }
}

/// `ftCommon_8007F76C` as the `x21F8` callback.
unsafe extern "C" fn cb_ftCommon_8007F76C(gobj: *mut HSD_GObj) {
    melee_fighter::ftcommon2::ftcommon_8007f76c(&mut *fpp(gobj));
}

pub fn ftPp_SpecialS_80120E68(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        let gobj2 = ext::Player_GetEntityAtIndex((*fp).player_id as i32, 1);

        if !gobj2.is_null() {
            let fp2 = fpp(gobj2);
            (*fp).self_vel.x = (*fp2).cur_pos.x - (*fp).cur_pos.x;
            (*fp).self_vel.y = (*fp2).cur_pos.y - (*fp).cur_pos.y;
            (*fp).self_vel.z = 0.0;
            (*fp).self_vel.x = -fmadds(3.0, (*fp2).facing_dir, -(*fp).self_vel.x);
            (*fp).self_vel.y += 5.0;
            melee_runtime::lbvector::normalize(&mut *(&mut (*fp).self_vel as *mut Vec as *mut melee_core::vec::Vec3));
            let dx = ((*fp).cur_pos.x - (*fp2).cur_pos.x) * ((*fp).cur_pos.x - (*fp2).cur_pos.x);
            let dy = ((*fp).cur_pos.y - (*fp2).cur_pos.y) * ((*fp).cur_pos.y - (*fp2).cur_pos.y);
            let dist = crate::pure::my_sqrtf(dx + dy) / (*da).x98;
            (*fp).self_vel.x *= (*da).x94 + dist;
            (*fp).self_vel.y *= (*da).x94 + dist;
            if (*fp).self_vel.x > 0.0 {
                (*fp).facing_dir = 1.0;
            } else {
                (*fp).facing_dir = -1.0;
            }
        }
    }
}

/// The C function falls out of its `goto end` without a value when the string item is missing;
/// every caller discards the result, so that path returns false here.
pub fn ftPp_SpecialS_80120FE0(gobj: G) -> i32 {
    unsafe {
        let fp = fpp(gobj);
        let cmd = (*fp).mv.pp.speciallw.x0;

        if cmd > 8 && cmd <= 0x53 {
            let item_gobj = (*fp).u.pp.x2238;
            if !item_gobj.is_null() {
                let ip = (*item_gobj).user_data as *mut Item;
                let sa = (*(*ip).xC4_article_data).x4_specialAttributes as *const itClimbersStringAttributes;
                let ev0 = (*sa).x18;
                let ev1 = (*sa).x1C;
                let ev2 = (*sa).x20;
                if cmd == ev0 {
                    ext::it_802C3950(item_gobj);
                } else if cmd == ev1 {
                    ext::it_802C3810(item_gobj);
                } else if cmd == ev2 {
                    ext::it_802C3864(item_gobj);
                }
                if (*fp).mv.pp.speciallw.x0 == 0x53 {
                    ext::it_802C2750((*fp).u.pp.x2238);
                }
            } else {
                return 0;
            }
        } else if (*fp).mv.pp.speciallw.x0 == 8 {
            ftPp_SpecialS_801210C8(gobj);
            if (*fp).u.pp.x2238.is_null() {
                st::ft_8008A2BC(gobj);
                return 1;
            }
        }
        0
    }
}

pub fn ftPp_SpecialS_801210C8(arg0: G) {
    unsafe {
        let fp = fpp(arg0);
        let mut sp10: Vec3 = core::mem::zeroed();
        ext::lb_8000B1CC((*(*fp).parts.add(FtPart_L4thNb as usize)).joint, core::ptr::null(), &mut sp10);
        let dir = (*fp).facing_dir;
        (*fp).u.pp.x2238 = ext::it_802C27D4(arg0, &mut sp10, (*fp).motion_id as u32, dir);
        (*fp).x1984_heldItemSpec = (*fp).u.pp.x2238;
        if !(*fp).u.pp.x2238.is_null() {
            (*fp).death3_cb = Some(crate::chars::cb::ftpopo__ftPp_Init_8011F060);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftpopo__ftPp_Init_8011F060);
        }
    }
}

pub fn ftPp_SpecialS_80121164(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.pp.x2238.is_null() {
            ext::it_802C2750((*fp).u.pp.x2238);
            ftPp_SpecialS_8012114C(gobj);
        }
    }
}

/// The tail shared by both Enter functions.
unsafe fn enter_tail(gobj: G) {
    let mut fp = fpp(gobj);
    (*fp).cmd_vars[2] = 0;
    (*fp).cmd_vars[1] = 0;
    (*fp).cmd_vars[0] = 0;
    fp = fpp(gobj);
    (*fp).mv.pp.unk_80123954.x0 = 1;
    (*fp).u.pp.x223C = 0;
    (*fp).u.pp.x2240.z = 0.0;
    (*fp).u.pp.x2240.y = 0.0;
    (*fp).u.pp.x2240.x = 0.0;
}

pub fn ftPp_SpecialHi_Enter(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        (*fp).gr_vel /= (*da).x84;

        ftPp_SpecialHi_801218AC(gobj);
        enter_tail(gobj);
    }
}

pub fn ftPp_SpecialAirHi_Enter(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        (*fp).self_vel.x /= (*da).x84;
        (*fp).self_vel.y /= (*da).x88;

        ftPp_SpecialHi_801218F8(gobj);

        (*fp).x1968_jumpsUsed = crate::chars::caps_max_jumps(&*fp) as u8;
        enter_tail(gobj);
    }
}

/// `checkNanaInRange` (static inline).
unsafe fn checkNanaInRange(gobj: G) -> bool {
    let fp = fpp(gobj);
    let da = da(fp);
    let nana_gobj = ext::Player_GetEntityAtIndex((*fp).player_id as i32, 1);
    if !nana_gobj.is_null() {
        let nana_pos = &(*fpp(nana_gobj)).cur_pos;
        let dx = ((*fp).cur_pos.x - nana_pos.x) * ((*fp).cur_pos.x - nana_pos.x);
        let dy = ((*fp).cur_pos.y - nana_pos.y) * ((*fp).cur_pos.y - nana_pos.y);
        if melee_runtime::gekko::sqrtf(dx + dy) < (*da).x7C && ext::ftNn_Init_8012300C(nana_gobj) == 1 {
            return true;
        }
    }
    false
}

/// `incrementMvAndCheck` (static inline).
unsafe fn incrementMvAndCheck(gobj: G) {
    let fp = fpp(gobj);
    (*fp).mv.pp.unk_80123954.x0 += 1;
    ftPp_SpecialS_80120FE0(gobj);
}

pub fn ftPp_SpecialHiStart_0_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[2] != 0 {
            (*fp).cmd_vars[2] = 0;
            if !checkNanaInRange(gobj) {
                ftPp_SpecialHi_80122098(gobj);
                return;
            }
            (*fp).set_x2222_b2(1);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ftPp_SpecialHi_80121DA0(gobj);
        } else {
            incrementMvAndCheck(gobj);
        }
    }
}

pub fn ftPp_SpecialAirHiStart_0_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[2] != 0 {
            (*fp).cmd_vars[2] = 0;
            if !checkNanaInRange(gobj) {
                ftPp_SpecialHi_801220D4(gobj);
                return;
            }
            (*fp).set_x2222_b2(1);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ftPp_SpecialHi_80121DD8(gobj);
            return;
        }
        incrementMvAndCheck(gobj);
    }
}

/// The IASA body shared by the ground and air Start_0 states.
unsafe fn start0_iasa(gobj: G) {
    let fp = fpp(gobj);
    let da = da(fp);
    if (*fp).cmd_vars[0] != 0 {
        (*fp).cmd_vars[0] = 0;
        if abs_c((*fp).input.lstick[0].x) > (*da).x80 {
            st::ftCommon_UpdateFacing(&mut *fp);
            ext::ftPartSetRotY(fp, 0, (1.5707963267948966_f64 * (*fp).facing_dir as f64) as f32);
        }
    }
}

pub fn ftPp_SpecialHiStart_0_IASA(gobj: G) {
    unsafe { start0_iasa(gobj) }
}

pub fn ftPp_SpecialAirHiStart_0_IASA(gobj: G) {
    unsafe { start0_iasa(gobj) }
}

/// Stores Nana's hand position (when she is in a Belay state) in `fp->u.pp.x2240`.
unsafe fn store_nana_hand(gobj: G, fp: *mut Fighter) {
    let mut sp: Vec3 = core::mem::zeroed();
    sp.x = 0.0;
    sp.y = 0.0;
    sp.z = 0.0;
    let nn_gobj = ext::Player_GetEntityAtIndex((*fpp(gobj)).player_id as i32, 1);
    if !nn_gobj.is_null() {
        let nn_fp = fpp(nn_gobj);
        if (*nn_fp).motion_id >= ftPp_MS_SpecialHi_0 as _ && (*nn_fp).motion_id <= ftPp_MS_SpecialHi_5 as _ {
            ext::lb_8000B1CC((*(*nn_fp).parts.add(FtPart_L4thNb as usize)).joint, core::ptr::null(), &mut sp);
        }
    }
    (*fp).u.pp.x2240 = sp;
}

pub fn ftPp_SpecialHiStart_0_Phys(gobj: G) {
    unsafe {
        st::ft_80084F3C(gobj);
        let fp = fpp(gobj);
        store_nana_hand(gobj, fp);
    }
}

/// `doFallPhys` (static inline) / the first half of the air Start_0 Phys.
unsafe fn doFallPhys(gobj: G) {
    let fp = fpp(gobj);
    let da = da(fp);
    co::fall(&mut *fp, (*da).x8C, (*da).x90);
    co::calc_self_accel_deaccel_air(&mut *fp);
}

pub fn ftPp_SpecialAirHiStart_0_Phys(gobj: G) {
    unsafe {
        doFallPhys(gobj);
        let fp = fpp(gobj);
        store_nana_hand(gobj, fp);
    }
}

/// The cmd-var-1 poll shared by both Throw_0 Anim states: true when Nana is caught.
unsafe fn throw0_nana_poll(gobj: G) -> bool {
    let fp = fpp(gobj);
    if (*fp).cmd_vars[1] != 0 {
        (*fp).cmd_vars[1] = 0;
        let nn_gobj = ext::Player_GetEntityAtIndex((*fpp(gobj)).player_id as i32, 1);
        let found = if !nn_gobj.is_null() && ext::ftNn_Init_8012309C(nn_gobj) == 1 { 1 } else { 0 };
        if found == 1 {
            ftPp_SpecialHi_8012280C(gobj);
            return true;
        }
    }
    false
}

pub fn ftPp_SpecialHiThrow_0_Anim(gobj: G) {
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
        if throw0_nana_poll(gobj) {
            return;
        }
        let fp = fpp(gobj);
        (*fp).mv.pp.unk_80123954.x0 += 1;
        ftPp_SpecialS_80120FE0(gobj);
    }
}

pub fn ftPp_SpecialAirHiThrow_0_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_80096900(gobj, 0, 1, 0, (*da).x74, (*da).x78);
        }
        if throw0_nana_poll(gobj) {
            return;
        }
        let fp = fpp(gobj);
        (*fp).mv.pp.unk_80123954.x0 += 1;
        ftPp_SpecialS_80120FE0(gobj);
    }
}

pub fn ftPp_SpecialHiThrow_0_IASA(_gobj: G) {}
pub fn ftPp_SpecialAirHiThrow_0_IASA(_gobj: G) {}

pub fn ftPp_SpecialHiThrow_0_Phys(gobj: G) {
    unsafe {
        st::ft_80084F3C(gobj);
        let fp = fpp(gobj);
        store_nana_hand(gobj, fp);
    }
}

pub fn ftPp_SpecialAirHiThrow_0_Phys(gobj: G) {
    unsafe {
        doFallPhys(gobj);
        let fp = fpp(gobj);
        store_nana_hand(gobj, fp);
    }
}

pub fn ftPp_SpecialHiStart_1_Anim(gobj: G) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftPp_SpecialHi_80122348(gobj);
    }
}

pub fn ftPp_SpecialHiStart_1_IASA(_gobj: G) {}
pub fn ftPp_SpecialAirHiStart_1_IASA(_gobj: G) {}

pub fn ftPp_SpecialAirHiStart_1_Phys(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        co::fall(&mut *fp, (*da).xA8, (*da).xAC);
        if (*fp).self_vel.y < 0.0 {
            co::calc_self_accel_deaccel_air(&mut *fp);
        }
    }
}

pub fn ftPp_SpecialHiThrow_1_IASA(_gobj: G) {}
pub fn ftPp_SpecialAirHiThrow_1_IASA(_gobj: G) {}

pub fn ftPp_SpecialHi_80122348(arg0: G) {
    st::Fighter_ChangeMotionState(arg0, 0x15F, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
}

pub fn ftPp_SpecialHiThrow2_Anim(gobj: G) {
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        } else {
            let fp = fpp(gobj);
            (*fp).mv.pp.unk_80123954.x0 += 1;
            ftPp_SpecialS_80120FE0(gobj);
        }
    }
}

pub fn ftPp_SpecialAirHiThrow2_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_80096900(gobj, 0, 1, 0, (*da).x74, (*da).x78);
        } else {
            let fp2 = fpp(gobj);
            (*fp2).mv.pp.unk_80123954.x0 += 1;
            ftPp_SpecialS_80120FE0(gobj);
        }
    }
}

pub fn ftPp_SpecialHiThrow2_IASA(_gobj: G) {}
pub fn ftPp_SpecialAirHiThrow2_IASA(_gobj: G) {}

pub fn ftPp_SpecialHiThrow2_Phys(gobj: G) {
    unsafe {
        st::ft_80084F3C(gobj);
        let fp = fpp(gobj);
        store_nana_hand(gobj, fp);
    }
}

pub fn ftPp_SpecialAirHiThrow2_Phys(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        co::fall(&mut *fp, (*da).x9C, (*da).xA0);
        if abs_c((*fp).input.lstick[0].x) > (*da).x80 {
            co::calc_self_accel_drift_simple(
                &mut *fp,
                0.0,
                (*fp).co_attrs.air_drift_stick_mul * (*da).xB0,
                (*fp).co_attrs.air_drift_max * (*da).xB4,
            );
        } else if (*fp).self_vel.y < 0.0 {
            co::calc_self_accel_deaccel_air(&mut *fp);
        }
        let fp = fpp(gobj);
        store_nana_hand(gobj, fp);
    }
}

pub fn ftPp_SpecialAirHiThrow2_Coll(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        let cd = &mut (*fp).coll_data;
        if st::ft_CheckGroundAndLedge(gobj, ftGetFacingDirInt(fp)) != 0 {
            st::ftCo_LandingFallSpecial_Enter(gobj, 0, (*da).x78);
        } else if st::ftCliffCommon_80081298(gobj) == 0 {
            if (cd.env_flags & 0x3F) != 0 && (*fp).self_vel.x > 0.0 {
                (*fp).self_vel.x = 0.0;
            } else if (cd.env_flags & 0xFC0) != 0 && (*fp).self_vel.x < 0.0 {
                (*fp).self_vel.x = 0.0;
            } else if (cd.env_flags & 0x6000) != 0 {
                (*fp).self_vel.y = 0.0;
                st::ftCo_80096900(gobj, 0, 1, 0, (*da).x74, (*da).x78);
            }
        }
    }
}

pub fn ftPp_SpecialHi_8012280C(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Ground {
            st::ftCommon_8007D60C(&mut *fp);
        } else {
            (*fp).x1968_jumpsUsed = crate::chars::caps_max_jumps(&*fp) as u8;
        }
        ftPp_SpecialS_80120E68(gobj);
        st::Fighter_ChangeMotionState(gobj, 0x162, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).x21F8 = Some(cb_ftCommon_8007F76C);
    }
}
