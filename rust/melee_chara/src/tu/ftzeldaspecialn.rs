//! src/melee/ft/kinds/ftZelda/ftzeldaspecialn.c: Zelda's neutral B (Din's Fire reflect).
//! The functions not ported elsewhere are checked bit for bit against the C by a state oracle
//! (tests/tu_ftzeldaspecialn.rs over gen/turef/tu_ftzeldaspecialn_ref.c, shipping contraction).
//! Fused sites: none (tools/rust-port/fma_sites.py).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

/// Already ported (round 6 / round 12): re-exported under this TU.



/// Calls into TUs that are not ported yet (or into group 3, via the same fallback as `stubs`).
/// Each is a settable hook; an unset hook without a fallback panics.
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
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    /// Like `hook!`, but an unset hook runs the native port `$port` (group 3, already ported).
    macro_rules! hook_or {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:expr) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
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
            None => { unsafe { melee_world::tu::efsync::efSync_Spawn(gfx_id as _, gobj as _, &[joint as *const core::ffi::c_void]) as _ } },
        }
    }
    // `ftColl_CreateReflectHit(gobj, desc, on_reflect)` (ft/ftcoll.c).
    crate::port_hook!(FTCOLL_CREATEREFLECTHIT, ftColl_CreateReflectHit, crate::adapt::ftColl_CreateReflectHit, (g: *mut HSD_GObj, desc: *mut ReflectDesc, cb: HSD_GObjEvent));
    // Group 3 helpers the TU calls directly (ft/ftcommon.c).
    hook_or!(FTCOMMON_FALL, ftCommon_Fall, (fp: &mut Fighter, gravity: f32, terminal_vel: f32),
        super::fall_port);
    hook_or!(FTCOMMON_8007D5D4, ftCommon_8007D5D4, (fp: &mut Fighter), super::d5d4_port);
    hook_or!(FTCOMMON_8007D7FC, ftCommon_8007D7FC, (fp: &mut Fighter), super::d7fc_port);
}

/// Group 3's verified ports (melee_fighter), the unset-hook defaults above.
fn fall_port(fp: &mut Fighter, gravity: f32, terminal_vel: f32) {
    melee_fighter::ftcommon::fall(fp, gravity, terminal_vel)
}
fn d5d4_port(fp: &mut Fighter) {
    melee_fighter::ftcommon2::ftcommon_8007d5d4(fp)
}
fn d7fc_port(fp: &mut Fighter) {
    melee_fighter::ftcommon3::ftcommon_8007d7fc(fp)
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

/// `Fighter_SetEffectHitlagCallbacks(fp)` (ft/inlines.h).
#[inline(always)]
unsafe fn set_effect_hitlag_cbs(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

/// `ftCommon_GroundToAirStateChange(gobj, fp, msid, flags)` (ft/kinds/ftCommon/inlines.h).
#[inline(always)]
unsafe fn ground_to_air(gobj: *mut HSD_GObj, fp: *mut Fighter, msid: i32, flags: u32) {
    ext::ftCommon_8007D5D4(&mut *fp);
    st::Fighter_ChangeMotionState(gobj, msid, flags, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
}

/// `ftCommon_AirToGroundStateChange(gobj, fp, msid, flags)` (ft/kinds/ftCommon/inlines.h).
#[inline(always)]
unsafe fn air_to_ground(gobj: *mut HSD_GObj, fp: *mut Fighter, msid: i32, flags: u32) {
    ext::ftCommon_8007D7FC(&mut *fp);
    st::Fighter_ChangeMotionState(gobj, msid, flags, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
}

/// `ftCommon_GroundAirColl_MF` (ft/kinds/ftCommon/forward.h).
pub const ftCommon_GroundAirColl_MF: u32 = (Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

/// `ftZd_MF_SpecialN_Coll` (ft/kinds/ftZelda/forward.h): GroundAirColl | KeepGfx |
/// KeepColAnimHitStatus | SkipHit.
pub const ftZd_MF_SpecialN_Coll: u32 = ftCommon_GroundAirColl_MF
    | Ft_MF_KeepGfx as u32
    | Ft_MF_KeepColAnimHitStatus as u32
    | Ft_MF_SkipHit as u32;

/// `ftZd_SpecialN_8013A830` (accessory callback; stored into `accessory4_cb`).
pub unsafe extern "C" fn ftZd_SpecialN_8013A830(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialN_8013A830");
    let fp = fpp(gobj);
    if (*fp).x2219_b0() == 0 {
        ext::efSync_Spawn(1268, gobj, joint(fp, FtPart_TransN as usize));
        (*fp).set_x2219_b0(1);
    }
    set_effect_hitlag_cbs(fp);
    (*fp).accessory4_cb = None;
}

/// `ftZd_SpecialN_8013A8AC` (accessory callback; the air variant's).
pub unsafe extern "C" fn ftZd_SpecialN_8013A8AC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialN_8013A8AC");
    let fp = fpp(gobj);
    if (*fp).x2219_b0() == 0 {
        ext::efSync_Spawn(1269, gobj, joint(fp, FtPart_TransN as usize));
        (*fp).set_x2219_b0(1);
    }
    set_effect_hitlag_cbs(fp);
    (*fp).accessory4_cb = None;
}

/// `ftZd_SpecialN_8013ADB0` (the reflect-hit callback; the body is in chars::r12).
// never inlined: tests compare this callback by address, and an empty body would be copied per crate
#[inline(never)]
pub unsafe extern "C" fn ftZd_SpecialN_8013ADB0_cb(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialN_8013ADB0_cb");
    crate::chars::r12::ftZelda::ftzeldaspecialn::ftZd_SpecialN_8013ADB0(gobj)
}

/// `ftZd_Init_801393AC` (ft/kinds/ftZelda/ftzelda.c): death / take-damage callback, not ported yet.
pub unsafe extern "C" fn ftZd_Init_801393AC(_gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_Init_801393AC");
    { unsafe { crate::tu::ftzelda::ftZd_Init_801393AC(_gobj as _); } return; };
}

/// `startActionHelper(gobj)` (static, ftzeldaspecialn.c).
unsafe fn start_action_helper(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let attributes = da_of(fp);
    (*fp).cmd_vars[0] = 0;
    (*fp).mv.zd.specialn.x0 = (*attributes).x4;
}

/// `ftZd_SpecialN_Enter` (ftzeldaspecialn.c, 8013A7xx block).
pub fn ftZd_SpecialN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialN_Enter");
    unsafe {
        let temp_f1: f32 = 0.0;
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(gobj, 341, 0, temp_f1, 1.0, temp_f1, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        start_action_helper(gobj);
        (*fp).accessory4_cb = Some(ftZd_SpecialN_8013A830);
    }
}

/// `ftZd_SpecialAirN_Enter`.
pub fn ftZd_SpecialAirN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialAirN_Enter");
    unsafe {
        let fp = fpp(gobj);
        let sa = da_of(fp);
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x = (*fp).self_vel.x / (*sa).x8;
        st::Fighter_ChangeMotionState(gobj, 342, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        start_action_helper(gobj);
        (*fp).accessory4_cb = Some(ftZd_SpecialN_8013A8AC);
    }
}

/// Shared head of `ftZd_SpecialN_Anim` / `ftZd_SpecialAirN_Anim`: the reflect window.
unsafe fn reflect_window(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let attributes = da_of(fp);
    if (*fp).cmd_vars[0] == 1 {
        (*fp).cmd_vars[0] = 2;
        ext::ftColl_CreateReflectHit(gobj, &mut (*attributes).x84, Some(ftZd_SpecialN_8013ADB0_cb));
    }
    if (*fp).cmd_vars[0] == 0 {
        (*fp).set_reflecting(0);
    }
}

/// `ftZd_SpecialN_Anim`.
pub fn ftZd_SpecialN_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialN_Anim");
    unsafe {
        reflect_window(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

/// `ftZd_SpecialAirN_Anim`: falls into `ftCo_Fall_Enter` when the animation ends.
pub fn ftZd_SpecialAirN_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialAirN_Anim");
    unsafe {
        reflect_window(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

/// `ftZd_SpecialN_Coll`.
pub fn ftZd_SpecialN_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialN_Coll");
    unsafe {
        if st::ft_80082708(gobj) == 0 {
            ftZd_SpecialN_8013AC88(gobj);
        }
    }
}

/// `ftZd_SpecialAirN_Coll`.
pub fn ftZd_SpecialAirN_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialAirN_Coll");
    unsafe {
        if st::ft_80081D0C(gobj) != 0 {
            ftZd_SpecialN_8013AD1C(gobj);
        }
    }
}

/// `ftZd_SpecialN_8013AC88`: ground-to-air transition, reflect hit if the window is open.
pub fn ftZd_SpecialN_8013AC88(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialN_8013AC88");
    unsafe {
        let fp = fpp(gobj);
        ground_to_air(gobj, fp, 342, ftZd_MF_SpecialN_Coll);
        let attributes = da_of(fp);
        if (*fp).cmd_vars[0] == 2 {
            ext::ftColl_CreateReflectHit(gobj, &mut (*attributes).x84, Some(ftZd_SpecialN_8013ADB0_cb));
        }
        (*fp).accessory4_cb = Some(ftZd_SpecialN_8013A8AC);
    }
}

/// `ftZd_SpecialN_8013AD1C`: air-to-ground transition.
pub fn ftZd_SpecialN_8013AD1C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialN_8013AD1C");
    unsafe {
        let fp = fpp(gobj);
        air_to_ground(gobj, fp, 341, ftZd_MF_SpecialN_Coll);
        let attributes = da_of(fp);
        if (*fp).cmd_vars[0] == 2 {
            ext::ftColl_CreateReflectHit(gobj, &mut (*attributes).x84, Some(ftZd_SpecialN_8013ADB0_cb));
        }
        (*fp).accessory4_cb = Some(ftZd_SpecialN_8013A830);
    }
}

/// `ftZd_SpecialAirN_Phys` (8013AB9C): the reflect window's remaining frames, then Fall.
pub fn ftZd_SpecialAirN_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialAirN_Phys");
    unsafe {
        let fp = fpp(gobj);
        let var1 = (*fp).mv.zd.specialn.x0;
        let sa = da_of(fp);
        let terminal = (*fp).co_attrs.terminal_velocity;
        if var1 != 0 {
            (*fp).mv.zd.specialn.x0 = var1 - 1;
        } else {
            ext::ftCommon_Fall(&mut *fp, (*sa).xC, terminal);
        }
        melee_fighter::ftcommon::calc_self_accel_deaccel_quick_air(&mut *fp);
        st::ftColl_8007AEF8(gobj);
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftZd_SpecialN_IASA` (src/melee/ft/kinds/ftZelda/ftzeldaspecialn.c).
pub fn ftZd_SpecialN_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialN_IASA");
    unsafe {
        return;
    }
}

/// `ftZd_SpecialAirN_IASA` (src/melee/ft/kinds/ftZelda/ftzeldaspecialn.c).
pub fn ftZd_SpecialAirN_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialAirN_IASA");
    unsafe {
        return;
    }
}

/// `ftZd_SpecialN_Phys` (src/melee/ft/kinds/ftZelda/ftzeldaspecialn.c).
pub fn ftZd_SpecialN_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialN_Phys");
    unsafe {
        st::ft_80084F3C(gobj);
        st::ftColl_8007AEF8(gobj);
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftZd_SpecialN_8013ADB0` (src/melee/ft/kinds/ftZelda/ftzeldaspecialn.c).
pub fn ftZd_SpecialN_8013ADB0(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftZd_SpecialN_8013ADB0");
    unsafe {
        return;
    }
}
