//! src/melee/ft/kinds/ftKoopa/ftkoopaspecials.c: Bowser's Koopa Klaw (side B).
//! No fused sites (tools/rust-port/fma_sites.py); the only float expression is the sign
//! multiply in `ftKp_SpecialS_set_dir`. Smoke-tested by tests/tu_ftkoopaspecials.rs.
//!
//! `fp->mv.kp.unk1.x0/x4/x8/xC` and `fp->mv.kp.specials.b_held/x4/facing_dir/xC` are the same
//! four 32-bit words on the GameCube; `unk1` declares pointers at x0 and x8, which are 8 bytes
//! natively and would move x4/xC. The C only ever stores 0/1 through them, so every access here
//! goes through the `specials` view (b_held, x4, facing_dir, xC) to keep the C's aliasing.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2, 5, 7, 13): re-exported under this TU.
pub use crate::chars::r13::ftKoopa::ftkoopaspecials::{ftKp_SpecialSEndB_Phys, ftKp_SpecialSEndF_Phys, ftKp_SpecialSStart_Phys};
pub use crate::chars::r2::ftKoopa::ftkoopaspecials::{
    ftKp_SpecialAirSEndB_Phys, ftKp_SpecialAirSEndF_Phys, ftKp_SpecialAirSHit_Phys, ftKp_SpecialAirSStart_Anim,
    ftKp_SpecialAirSWait_Phys, ftKp_SpecialSHit_Phys, ftKp_SpecialSStart_Anim, ftKp_SpecialSWait_Phys,
};
pub use crate::chars::r5::ftKoopa::ftkoopaspecials::{
    ftKp_SpecialAirSEndB_Coll, ftKp_SpecialAirSEndF_Coll, ftKp_SpecialAirSHit_Coll, ftKp_SpecialAirSStart_Coll,
    ftKp_SpecialAirSWait_Coll, ftKp_SpecialS_80133324, ftKp_SpecialSEndB_Coll, ftKp_SpecialSEndF_Coll,
    ftKp_SpecialSHit_Coll, ftKp_SpecialSStart_Coll, ftKp_SpecialSWait_Coll,
};
pub use crate::chars::r7::ftKoopa::ftkoopaspecials::{ftKp_SpecialSWait_IASA_inline, inlineA0};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    /// `ftColl_8007ABD0(hit, flags, gobj)` (ft/ftcoll.c): shared with ftlinkattackair.
    pub use crate::tu::ftlinkattackair::ext::{ftColl_8007ABD0, FTCOLL_8007ABD0};
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
    pub static mut FTANIM_8006F0FC: Option<fn(gobj: *mut HSD_GObj, anim_rate: f32)> = None;
    /// `ftAnim_8006F0FC(gobj, rate)` (ft/ftanim.c): group 3's port unless a hook is installed.
    #[allow(non_snake_case)]
    pub fn ftAnim_8006F0FC(gobj: *mut HSD_GObj, anim_rate: f32) {
        match unsafe { FTANIM_8006F0FC } {
            Some(f) => f(gobj, anim_rate),
            None => unsafe { melee_fighter::tu::ftanim::ftAnim_8006F0FC(gobj, anim_rate) },
        }
    }
    // ftkoopa.c: the thrown-victim position follow-up of the End-B animation.
    pub static mut FTKP_SPECIALS_80132E30: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKp_SpecialS_80132E30(gobj: *mut HSD_GObj)  {
                match unsafe { FTKP_SPECIALS_80132E30 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkoopa::ftKp_SpecialS_80132E30(gobj as _); } return; },
                }
            }
}

/// Ft_MF_SkipMatAnim.
const transition_flags0: u32 = Ft_MF_SkipMatAnim as u32;

/// Ft_MF_SkipHit | SkipMatAnim | SkipColAnim | UpdateCmd | SkipItemVis | Unk19 |
/// SkipModelPartVis | SkipModelFlags | Unk27.
const transition_flags1: u32 = (Ft_MF_SkipHit
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

/// Ft_MF_SkipMatAnim | SkipColAnim | SkipItemVis | Unk19 | SkipModelPartVis | SkipModelFlags.
const transition_flags2: u32 = (Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags) as u32;

/// Ft_MF_SkipMatAnim | Ft_MF_Unk19.
const transition_flags3: u32 = (Ft_MF_SkipMatAnim | Ft_MF_Unk19) as u32;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftCo_800BC7E0` / `ftCo_800BC8D4` as `grabbed_cb` entries.
unsafe extern "C" fn cb_ftCo_800BC7E0(g: *mut HSD_GObj, vg: *mut HSD_GObj) {
    melee_fighter::tu::ftCo_CaptureKoopa::ftCo_800BC7E0(g, vg);
}
unsafe extern "C" fn cb_ftCo_800BC8D4(g: *mut HSD_GObj, vg: *mut HSD_GObj) {
    melee_fighter::tu::ftCo_CaptureKoopa::ftCo_800BC8D4(g, vg);
}

/// `ftCheckThrowB4` (ft/inlines.h): test and clear throw_flags_b4.
#[inline(always)]
fn ftCheckThrowB4(fp: &mut Fighter) -> bool {
    unsafe {
        if fp.anon_2210.anon_0.throw_flags_b4() != 0 {
            fp.anon_2210.anon_0.set_throw_flags_b4(0);
            true
        } else {
            false
        }
    }
}

/// `ftKp_SpecialS_set_dir` (static): the stick-flick direction (-1, 0, +1) while holding the
/// victim; also latches B and records the direction relative to the facing.
fn ftKp_SpecialS_set_dir(gobj: *mut HSD_GObj) -> i32 {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKoopaAttributes;
        let mut facing_dir: i32 = 0;
        if (*fp).input.pressed_buttons & (HSD_PAD_B as u32) != 0 {
            (*fp).mv.kp.specials.b_held = 1;
        }
        if (*fp).input.lstick[1].x > -(*da).x30 && (*fp).input.lstick[0].x < -(*da).x30 {
            facing_dir = -1;
        }
        if (*fp).input.lstick[1].x < (*da).x30 && (*fp).input.lstick[0].x > (*da).x30 {
            facing_dir = 1;
        }
        (*fp).mv.kp.specials.facing_dir = ((*fp).facing_dir * facing_dir as f32) as i32;
        facing_dir
    }
}

/// `ftCommon_8007E2D0(fp, 8, grab_cb, NULL, grabbed_cb)` then clear the two flags.
unsafe fn set_grab_callbacks(fp: *mut Fighter, grab_cb: unsafe extern "C" fn(*mut HSD_GObj), grabbed_cb: unsafe extern "C" fn(*mut HSD_GObj, *mut HSD_GObj)) {
    melee_fighter::ftcommon2::ftcommon_8007e2d0(&mut *fp, 8, Some(grab_cb), None, Some(grabbed_cb));
    (*fp).mv.kp.specials.b_held = 0;
    (*fp).mv.kp.specials.facing_dir = 0;
}

pub fn ftKp_SpecialS_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).anon_2210.throw_flags = 0;
        (*fp).cmd_vars[0] = 0;
        (*fp).mv.kp.specials.x4 = 0;
        (*fp).mv.kp.specials.xC = 0;
        st::Fighter_ChangeMotionState(gobj, 347, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        let fp = fpp(gobj);
        set_grab_callbacks(fp, cb_ftKp_SpecialS_8013302C, cb_ftCo_800BC7E0);
    }
}

pub fn ftKp_SpecialAirS_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).anon_2210.throw_flags = 0;
        (*fp).cmd_vars[0] = 0;
        (*fp).mv.kp.specials.x4 = 0;
        (*fp).mv.kp.specials.xC = 0;
        st::Fighter_ChangeMotionState(gobj, 353, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        let fp = fpp(gobj);
        set_grab_callbacks(fp, cb_ftKp_SpecialS_801330E4, cb_ftCo_800BC8D4);
    }
}

/// Enter the hit loop: `msid_loop` (or `msid_hit` when x4 is set) and arm the grab hitbox.
unsafe fn enter_hit(gobj: *mut HSD_GObj, fp: *mut Fighter, msid_hit: i32, msid_loop: i32) {
    if (*fp).mv.kp.specials.x4 != 0 {
        st::Fighter_ChangeMotionState(gobj, msid_hit, transition_flags0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    } else {
        st::Fighter_ChangeMotionState(gobj, msid_loop, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    }
    (*fp).set_x2222_b2(1);
    st::ftCommon_8007E2F4(&mut *fp, 511);
    st::ftCommon_8007E2FC(gobj);
    (*fp).mv.kp.specials.b_held = 0;
    (*fp).cmd_vars[0] = 0;
}

pub fn ftKp_SpecialS_8013302C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        enter_hit(gobj, fp, 349, 348);
    }
}

pub fn ftKp_SpecialS_801330E4(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        enter_hit(gobj, fp, 355, 354);
    }
}

pub fn ftKp_SpecialS_8013319C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftKp_MS_SpecialAirSStart as i32, transition_flags1);
        let fp = fpp(gobj);
        set_grab_callbacks(fp, cb_ftKp_SpecialS_801330E4, cb_ftCo_800BC8D4);
    }
}

pub fn ftKp_SpecialS_8013322C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp1 = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp1, 347, transition_flags1);
        let fp0 = fpp(gobj);
        set_grab_callbacks(fp0, cb_ftKp_SpecialS_8013302C, cb_ftCo_800BC7E0);
        (*fp1).mv.kp.specials.b_held = 0;
        (*fp1).cmd_vars[0] = 0;
    }
}

/// The ground Hit/Wait/End collision callbacks' shared body: abandon the hold and fall.
unsafe fn release_and_fall(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let victim_gobj = (*fp).victim_gobj;
    st::ftCommon_8007D5D4(&mut *fp);
    if !victim_gobj.is_null() {
        melee_fighter::tu::ftCo_CaptureCut::ftCo_800DC920(gobj, victim_gobj);
        st::ftCo_Fall_Enter(victim_gobj);
    }
    st::ftCo_Fall_Enter(gobj);
}

pub fn ftKp_SpecialS_801332C4(gobj: *mut HSD_GObj) {
    unsafe { release_and_fall(gobj) }
}

pub fn ftKp_SpecialS_80133398(gobj: *mut HSD_GObj) {
    unsafe { release_and_fall(gobj) }
}

pub fn ftKp_SpecialS_801333F8(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 350, transition_flags2);
        st::ftCommon_8007E2F4(&mut *fp, 511);
        st::ftCommon_8007E2FC(gobj);
        ext::ftAnim_8006F0FC(gobj, 0.0);
        (*fp).mv.kp.specials.b_held = 0;
        (*fp).cmd_vars[0] = 0;
    }
}

pub fn ftKp_SpecialS_80133484(gobj: *mut HSD_GObj) {
    unsafe { release_and_fall(gobj) }
}

/// Air End-F/End-B to the ground thrown-victim states: flip the facing around the state change
/// when the throw direction flag (xC) is set.
unsafe fn air_end_to_ground(gobj: *mut HSD_GObj, msid: i32, victim_msid: i32) {
    let fp = fpp(gobj);
    st::ftCommon_8007D7FC(&mut *fp);
    if (*fp).mv.kp.specials.xC != 0 {
        (*fp).facing_dir = -(*fp).facing_dir;
    }
    st::Fighter_ChangeMotionState(gobj, msid, transition_flags1, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    if (*fp).mv.kp.specials.xC != 0 {
        (*fp).facing_dir = -(*fp).facing_dir;
    }
    if !(*fp).victim_gobj.is_null() {
        melee_fighter::tu::ftCo_ThrownKoopa::ftCo_800BCE64((*fp).victim_gobj, victim_msid);
    }
    st::ftCommon_8007E2F4(&mut *fp, 511);
    st::ftCommon_8007E2FC(gobj);
}

pub fn ftKp_SpecialS_801334E4(gobj: *mut HSD_GObj) {
    unsafe { air_end_to_ground(gobj, 351, ftCo_MS_ThrownKoopaF as i32) }
}

pub fn ftKp_SpecialS_8013359C(gobj: *mut HSD_GObj) {
    unsafe { air_end_to_ground(gobj, 352, ftCo_MS_ThrownKoopaB as i32) }
}

/// Fighter callback entries for the static-address helpers above.
pub unsafe extern "C" fn cb_ftKp_SpecialS_8013302C(g: *mut HSD_GObj) {
    ftKp_SpecialS_8013302C(g);
}
pub unsafe extern "C" fn cb_ftKp_SpecialS_801330E4(g: *mut HSD_GObj) {
    ftKp_SpecialS_801330E4(g);
}
pub unsafe extern "C" fn cb_ftKp_SpecialS_8013319C(g: *mut HSD_GObj) {
    ftKp_SpecialS_8013319C(g);
}
pub unsafe extern "C" fn cb_ftKp_SpecialS_8013322C(g: *mut HSD_GObj) {
    ftKp_SpecialS_8013322C(g);
}
pub unsafe extern "C" fn cb_ftKp_SpecialS_801332C4(g: *mut HSD_GObj) {
    ftKp_SpecialS_801332C4(g);
}
pub unsafe extern "C" fn cb_ftKp_SpecialS_80133398(g: *mut HSD_GObj) {
    ftKp_SpecialS_80133398(g);
}
pub unsafe extern "C" fn cb_ftKp_SpecialS_801333F8(g: *mut HSD_GObj) {
    ftKp_SpecialS_801333F8(g);
}
pub unsafe extern "C" fn cb_ftKp_SpecialS_80133484(g: *mut HSD_GObj) {
    ftKp_SpecialS_80133484(g);
}
pub unsafe extern "C" fn cb_ftKp_SpecialS_801334E4(g: *mut HSD_GObj) {
    ftKp_SpecialS_801334E4(g);
}
pub unsafe extern "C" fn cb_ftKp_SpecialS_8013359C(g: *mut HSD_GObj) {
    ftKp_SpecialS_8013359C(g);
}

pub fn ftKp_SpecialSHit_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp1 = fpp(gobj);
        let koopa_attr = (*fp1).dat_attrs as *const ftKoopaAttributes;
        if (*fp1).cmd_vars[0] != 0 && (*fp1).mv.kp.specials.x4 != 0 {
            ext::ftColl_8007ABD0(&mut (*fp1).x914[0], (*koopa_attr).x2C, gobj);
            (*fp1).cmd_vars[0] = 0;
        }
        if st::ftAnim_IsFramesRemaining(gobj) != 0 {
            return;
        }
        if (*fp1).mv.kp.specials.b_held != 0 {
            (*fp1).mv.kp.specials.x4 = 1;
            let fp0 = fpp(gobj);
            enter_hit(gobj, fp0, 349, 348);
            melee_fighter::tu::ftCo_CaptureDamageKoopa::ftCo_800BC9C8((*fp1).victim_gobj);
        } else {
            let fp0 = fpp(gobj);
            st::Fighter_ChangeMotionState(gobj, 350, transition_flags3, (*fp0).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
            ext::ftAnim_8006F0FC(gobj, 0.0);
            (*fp0).mv.kp.specials.b_held = 0;
            (*fp0).cmd_vars[0] = 0;
            st::ftCommon_8007E2F4(&mut *fp0, 511);
        }
    }
}

/// `inlineA1` (static inline): the air Hit0 -> Hit0_1 switch when B was released.
pub fn inlineA1(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(gobj, ftKp_MS_SpecialAirSHit0_1 as i32, transition_flags3, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_8006F0FC(gobj, 0.0);
        (*fp).mv.kp.specials.b_held = 0;
        (*fp).cmd_vars[0] = 0;
        st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
    }
}

pub fn ftKp_SpecialAirSHit_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKoopaAttributes;
        if (*fp).cmd_vars[0] != 0 && (*fp).mv.kp.specials.x4 != 0 {
            ext::ftColl_8007ABD0((*fp).x914.as_mut_ptr(), (*da).x2C, gobj);
            (*fp).cmd_vars[0] = 0;
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if (*fp).mv.kp.specials.b_held != 0 {
                (*fp).mv.kp.specials.x4 = 1;
                inlineA0(gobj);
                melee_fighter::tu::ftCo_CaptureDamageKoopa::ftCo_800BCAF4((*fp).victim_gobj);
            } else {
                inlineA1(gobj);
            }
        }
    }
}

pub fn ftKp_SpecialSWait_Anim(_gobj: *mut HSD_GObj) {}

pub fn ftKp_SpecialAirSWait_Anim(_gobj: *mut HSD_GObj) {}

/// `doEndFAnim` (static inline): flip the facing on the script's B4 pulse, release the victim on
/// the cmd_vars[0] pulse, then `cb` at the end of the animation.
fn doEndFAnim(gobj: *mut HSD_GObj, cb: fn(*mut HSD_GObj)) {
    unsafe {
        let fp = fpp(gobj);
        if ftCheckThrowB4(&mut *fp) {
            (*fp).facing_dir = -(*fp).facing_dir;
            (*fp).mv.kp.specials.xC = 1;
        }
        if (*fp).cmd_vars[0] != 0 {
            let victim_gobj = (*fp).victim_gobj;
            if !victim_gobj.is_null() {
                st::ftCommon_8007E2F4(&mut *fp, 0);
                melee_fighter::tu::ftCo_Throw::ftCo_800DE2A8(gobj, victim_gobj);
                melee_fighter::tu::ftCo_Thrown::ftCo_800DE7C0(victim_gobj, core::ptr::null_mut(), false);
                (*fp).cmd_vars[0] = 0;
            }
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            cb(gobj);
        }
    }
}

/// `doEndBAnim` (static inline).
fn doEndBAnim(gobj: *mut HSD_GObj, cb: fn(*mut HSD_GObj)) {
    ext::ftKp_SpecialS_80132E30(gobj);
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        cb(gobj);
    }
}

pub fn ftKp_SpecialSEndF_Anim(gobj: *mut HSD_GObj) {
    doEndFAnim(gobj, st::ft_8008A2BC);
}

pub fn ftKp_SpecialSEndB_Anim(gobj: *mut HSD_GObj) {
    doEndBAnim(gobj, st::ft_8008A2BC);
}

pub fn ftKp_SpecialAirSEndF_Anim(gobj: *mut HSD_GObj) {
    doEndFAnim(gobj, st::ftCo_Fall_Enter);
}

pub fn ftKp_SpecialAirSEndB_Anim(gobj: *mut HSD_GObj) {
    doEndBAnim(gobj, st::ftCo_Fall_Enter);
}

/// `ftKoopa_SpecialS_ChangeAction` (static inline): throw the victim forward or backward
/// depending on the flick direction recorded by `set_dir`.
fn ftKoopa_SpecialS_ChangeAction(gobj: *mut HSD_GObj, kp_msid_f: i32, victim_msid_f: i32, kp_msid_b: i32, victim_msid_b: i32) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).anon_2210.throw_flags = 0;
        (*fp).cmd_vars[0] = 0;
        if (*fp).mv.kp.specials.facing_dir == 1 {
            st::Fighter_ChangeMotionState(gobj, kp_msid_f, Ft_MF_Unk19 as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
            melee_fighter::tu::ftCo_ThrownKoopa::ftCo_800BCDE0((*fp).victim_gobj, victim_msid_f);
        } else {
            st::Fighter_ChangeMotionState(gobj, kp_msid_b, Ft_MF_Unk19 as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
            melee_fighter::tu::ftCo_ThrownKoopa::ftCo_800BCDE0((*fp).victim_gobj, victim_msid_b);
        }
    }
}

pub fn ftKp_SpecialSHit_IASA(gobj: *mut HSD_GObj) {
    let facing_dir = ftKp_SpecialS_set_dir(gobj);
    if facing_dir != 0 {
        ftKoopa_SpecialS_ChangeAction(
            gobj,
            ftKp_MS_SpecialSEndF as i32,
            ftCo_MS_ThrownKoopaF as i32,
            ftKp_MS_SpecialSEndB as i32,
            ftCo_MS_ThrownKoopaB as i32,
        );
    }
}

pub fn ftKp_SpecialAirSHit_IASA(gobj: *mut HSD_GObj) {
    let facing_dir = ftKp_SpecialS_set_dir(gobj);
    if facing_dir != 0 {
        ftKoopa_SpecialS_ChangeAction(
            gobj,
            ftKp_MS_SpecialAirSEndF as i32,
            ftCo_MS_ThrownKoopaAirF as i32,
            ftKp_MS_SpecialAirSEndB as i32,
            ftCo_MS_ThrownKoopaAirB as i32,
        );
    }
}

pub fn ftKp_SpecialSWait_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let facing_dir = ftKp_SpecialS_set_dir(gobj);
        if facing_dir != 0 {
            ftKoopa_SpecialS_ChangeAction(
                gobj,
                ftKp_MS_SpecialSEndF as i32,
                ftCo_MS_ThrownKoopaF as i32,
                ftKp_MS_SpecialSEndB as i32,
                ftCo_MS_ThrownKoopaB as i32,
            );
        } else if (*fp).mv.kp.specials.b_held != 0 {
            (*fp).mv.kp.specials.x4 = 1;
            ftKp_SpecialSWait_IASA_inline(gobj);
            melee_fighter::tu::ftCo_CaptureDamageKoopa::ftCo_800BC9C8((*fp).victim_gobj);
        }
    }
}

pub fn ftKp_SpecialAirSWait_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let facing_dir = ftKp_SpecialS_set_dir(gobj);
        if facing_dir != 0 {
            ftKoopa_SpecialS_ChangeAction(
                gobj,
                ftKp_MS_SpecialAirSEndF as i32,
                ftCo_MS_ThrownKoopaAirF as i32,
                ftKp_MS_SpecialAirSEndB as i32,
                ftCo_MS_ThrownKoopaAirB as i32,
            );
        } else if (*fp).mv.kp.specials.b_held != 0 {
            (*fp).mv.kp.specials.x4 = 1;
            inlineA0(gobj);
            melee_fighter::tu::ftCo_CaptureDamageKoopa::ftCo_800BCAF4((*fp).victim_gobj);
        }
    }
}

pub fn ftKp_SpecialAirSStart_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let vel_y = (*fp).self_vel.y;
        st::ft_80085134(gobj);
        (*fp).self_vel.y = vel_y;
        melee_fighter::ftcommon::fall_basic(&mut *fp);
    }
}
