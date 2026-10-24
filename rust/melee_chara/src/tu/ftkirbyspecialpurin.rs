//! src/melee/ft/kinds/ftKirby/ftkirbyspecialpurin.c: Kirby's Rollout (Jigglypuff copy ability).
//! Fused sites (tools/rust-port/fma_sites.py): f32 `fmadds` in the charge-spin Anims
//! (`x14 += x34.x * (x2C * MTXDegToRad(spin))`, Loop/Full/AirLoop/AirFull), in `ftKb_PrSpecialN_Anim`
//! / `ftKb_PrSpecialNHit_Anim` (`x14 += spin_after * (f32)(0.2 * turn * -x34.x)`), 4 in
//! `ftKb_PrSpecialNTurn_Phys` (`gr_vel += scale * (x1C +- influence)`), 1 in `ftKb_PrWallBounceEffect`
//! (`y += 0.5f * ABS(top + bottom)`); one f64 fma in `ftKb_PrSpecialNTurn_Anim`
//! (`x14 += 0.2 * turn * -x34.x`, in double). `mv.pr.specialn.x2C` is an `int` in the C header.
//! The spin/charge arithmetic is checked against the C by tests/tu_ftkirbyspecialpurin.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

use crate::stubs as st;
use crate::tu::ftkirbyspecialn::{fpp, ftKb_SpecialN_set_cbs, G};
use melee_core::fp::fmadds;
use melee_fighter::hooks as fh;
use melee_runtime::lbtrigf::atan2f;
use melee_types::*;

/// Already ported (root module): the Fighter-only halves, wrapped below with the C signature.
use crate::chars::ftKirby::ftkirbyspecialpurin as half;

/// Calls into TUs that are not ported yet (or ported elsewhere: the default then calls the port).
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
    macro_rules! hook_or {
        ($hook:ident, $name:ident, $port:expr, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => ($port)($($a),*),
                }
            }
        };
    }
    type G = *mut HSD_GObj;
    pub use crate::tu::ftcaptainspecialn::ext::{efSync_Spawn, EFSYNC_SPAWN};
    pub use crate::tu::ftkirbyspecialmewtwo::ext::{ftCo_800BFFAC, ftCo_800BFFD0, ft_80088510, FTCO_800BFFAC, FTCO_800BFFD0, FT_80088510};
    pub static mut HSD_JOBJSETSCALE: Option<fn(*mut HSD_JObj, *const Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_JObjSetScale(j: *mut HSD_JObj, scale: *const Vec3)  {
                match unsafe { HSD_JOBJSETSCALE } {
                    Some(f) => f(j, scale),
                    None => { unsafe { let _ = crate::tu::ftinl::HSD_JObjSetScale(j as _, scale as _); } return; },
                }
            }
    pub static mut HSD_JOBJGETSCALE: Option<fn(*mut HSD_JObj, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_JObjGetScale(j: *mut HSD_JObj, scale: *mut Vec3)  {
                match unsafe { HSD_JOBJGETSCALE } {
                    Some(f) => f(j, scale),
                    None => { unsafe { let _ = crate::tu::ftinl::HSD_JObjGetScale(j as _, scale as _); } return; },
                }
            }
    pub static mut HSD_JOBJSETROTATIONY: Option<fn(*mut core::ffi::c_void, f32) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_JObjSetRotationY(j: *mut core::ffi::c_void, rot: f32)  {
                match unsafe { HSD_JOBJSETROTATIONY } {
                    Some(f) => f(j, rot),
                    None => { unsafe { let _ = melee_fighter::tu::ftCo_Lift::HSD_JObjSetRotationY(j as _, rot as _); } return; },
                }
            }
    crate::port_hook!(FTCOLL_8007AFF8, ftColl_8007AFF8, crate::adapt::ftColl_8007AFF8, (g: G));
    crate::port_hook!(FTCOLL_8007ABD0, ftColl_8007ABD0, crate::adapt::ftColl_8007ABD0, (hit: *mut HitCapsule, damage: u32, g: G));
    pub static mut MPLIB_800569EC: Option<fn(u32) -> f32> = None;
    #[allow(non_snake_case)]
    pub fn mpLib_800569EC(flags: u32) -> f32 {
        match unsafe { MPLIB_800569EC } {
            Some(f) => f(flags),
            None => { unsafe { melee_world::tu::mplib::mpLib_800569EC(flags as _) as _ } },
        }
    }
    crate::port_hook!(CAMERA_REQUESTQUAKE, Camera_RequestQuake, crate::adapt::Camera_RequestQuake, (kind: CmQuakeKind, pos: *const Vec3));
    crate::port_hook!(FT_800892A0, ft_800892A0, crate::adapt::ft_800892A0, (g: G));
    hook_or!(FTPARTSETROTX, ftPartSetRotX, |fp: *mut Fighter, i, v| melee_fighter::tu::ftparts::ftPartSetRotX(unsafe { &*fp }, i, v), (fp: *mut Fighter, part_idx: i32, rot: f32));
    hook_or!(FTPARTSETROTY, ftPartSetRotY, |fp: *mut Fighter, i, v| melee_fighter::tu::ftparts::ftPartSetRotY(unsafe { &*fp }, i, v), (fp: *mut Fighter, part_idx: i32, rot: f32));
    hook_or!(FTCOMMON_8007D5D4, ftCommon_8007D5D4, |fp: *mut Fighter| crate::stubs::ftCommon_8007D5D4(unsafe { &mut *fp }), (fp: *mut Fighter));
    hook_or!(FTCOMMON_8007D7FC, ftCommon_8007D7FC, |fp: *mut Fighter| crate::stubs::ftCommon_8007D7FC(unsafe { &mut *fp }), (fp: *mut Fighter));
    hook_or!(FTCOMMON_FALL, ftCommon_Fall, |fp: *mut Fighter, g, t| crate::stubs::ftCommon_Fall(unsafe { &mut *fp }, g, t), (fp: *mut Fighter, gravity: f32, terminal: f32));
    hook_or!(FTCOMMON_SETSELFMOVEMENT, ftCommon_SetSelfMovementFromGroundedMovement, |fp: *mut Fighter| melee_fighter::ftcommon::set_self_movement_from_grounded_movement(unsafe { &mut *fp }), (fp: *mut Fighter));
    hook_or!(FT_80089824, ft_80089824, melee_fighter::ft0892::ft_80089824, (g: G));
    hook_or!(FT_80082888, ft_80082888, melee_fighter::tu::ft_081b::ft_80082888, (g: G, b: *mut ftCollisionBox) -> bool);
    hook_or!(FT_80082978, ft_80082978, melee_fighter::tu::ft_081b::ft_80082978, (g: G, b: *mut ftCollisionBox) -> bool);
    hook_or!(FT_8008239C, ft_8008239C, melee_fighter::tu::ft_081b::ft_8008239c, (g: G, dir: i32, b: *mut ftCollisionBox) -> bool);
    hook_or!(FT_800824A0, ft_800824A0, melee_fighter::tu::ft_081b::ft_800824a0, (g: G, b: *mut ftCollisionBox) -> bool);
    hook_or!(FTCLIFFCOMMON_80081370, ftCliffCommon_80081370, melee_fighter::tu::ftcliffcommon::ftcliffcommon_80081370, (g: G));
}

/// `MTXDegToRad(a)` (dolphin/mtx.h): `a * 0.01745329252f`.
#[inline(always)]
pub fn MTXDegToRad(a: f32) -> f32 {
    a * 0.01745329252f32
}

const M_PI: f64 = core::f64::consts::PI;
const M_PI_2: f64 = core::f64::consts::FRAC_PI_2;

/// `/* 3CB710 */` and `/* 3CB720 */` scale tables; the air step reads `base[frame + 4]` off the end of
/// the first (the two are adjacent in the data), so they are one array here.
pub static ftKb_Init_803CB710: [f32; 8] = [0.65, 0.7, 0.8, 1.0, 1.1, 1.35, 1.3, 1.2];

/// `/* 3CB758 */ ftCollisionBox ftKb_Init_803CB758`.
pub static mut ftKb_Init_803CB758: ftCollisionBox =
    ftCollisionBox { top: 8.0, bottom: 0.0, left: Vec2 { x: -4.0, y: 4.0 }, right: Vec2 { x: 4.0, y: 4.0 } };

macro_rules! sn {
    ($fp:expr) => {
        (*$fp).mv.pr.specialn
    };
}

#[inline(always)]
unsafe fn da_of(fp: *mut Fighter) -> *const ftKb_DatAttrs {
    (*fp).dat_attrs as *const ftKb_DatAttrs
}

#[inline(always)]
unsafe fn jobj_of(gobj: G) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}

#[inline(always)]
fn abs_f(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

#[inline(always)]
fn signf(x: f32) -> f32 {
    if x > 0.0 {
        1.0
    } else {
        -1.0
    }
}

/// `ftPartSetRotY(fp, part, M_PI_2)`-style call: the double constant narrowed to the f32 parameter.
#[inline(always)]
unsafe fn rot_y(fp: *mut Fighter, part: i32, v: f64) {
    ext::ftPartSetRotY(fp, part, v as f32);
}

// Callback entries (extern "C") stored in Fighter fields.
pub unsafe extern "C" fn cb_fn_80100E0C(g: G) {
    fn_80100E0C(g)
}
pub unsafe extern "C" fn cb_fn_80105978(g: G) {
    fn_80105978(g)
}
pub unsafe extern "C" fn cb_fn_80100F60(g: G) {
    fn_80100F60(g)
}

// ---------------------------------------------------------------------------------------------
// Static inlines
// ---------------------------------------------------------------------------------------------

/// `ftPr_NormalizeAndSetRollAngle` (ftPurin/inlines.h): wrap the roll angle into [0, 2 pi].
pub fn ftPr_NormalizeAndSetRollAngle(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        while sn!(fp).x14 < 0.0 {
            sn!(fp).x14 = (sn!(fp).x14 as f64 + M_PI * 2.0) as f32;
        }
        while sn!(fp).x14 as f64 > M_PI * 2.0 {
            sn!(fp).x14 = (sn!(fp).x14 as f64 - M_PI * 2.0) as f32;
        }
        ext::ftPartSetRotX(fp, FtPart_YRotN as i32, sn!(fp).x14);
    }
}

/// `ftPr_SetFacingDir` (ftPurin/inlines.h).
pub fn ftPr_SetFacingDir(fp: *mut Fighter) {
    unsafe {
        if sn!(fp).facing_dir != 0.0 {
            (*fp).facing_dir = sn!(fp).facing_dir;
            sn!(fp).x34.x = (*fp).facing_dir;
        }
        sn!(fp).facing_dir = 0.0;
        sn!(fp).xC = 0;
    }
}

/// `ftPr_MirrorSpecialN` (ftPurin/inlines.h).
pub fn ftPr_MirrorSpecialN(fp: *mut Fighter) {
    unsafe {
        (*fp).self_vel.x = -(*fp).self_vel.x;
        (*fp).gr_vel = -(*fp).gr_vel;
        (*fp).xE4_ground_accel_1 = -(*fp).xE4_ground_accel_1;
        sn!(fp).x10 = -sn!(fp).x10;
        sn!(fp).x14 = -sn!(fp).x14;
        sn!(fp).x18 = -sn!(fp).x18;
        sn!(fp).x1C = -sn!(fp).x1C;
        sn!(fp).facing_dir = -sn!(fp).facing_dir;
        sn!(fp).x34.x = -sn!(fp).x34.x;
        sn!(fp).x34.y = -sn!(fp).x34.y;
    }
}

/// `ftKb_PrScaleAnimStep` / `ftKb_AirScaleAnimStep` (static inlines): squash-and-stretch for 4 frames.
pub fn ftKb_PrScaleAnimStep(gobj: G, scale: &mut Vec3) {
    unsafe {
        let fp = fpp(gobj);
        let jobj = jobj_of(gobj);
        let frame = sn!(fp).x8;
        if (0..4).contains(&frame) {
            scale.x = (*fp).u.kb.x8C.x;
            scale.y = (*fp).u.kb.x8C.y * ftKb_Init_803CB710[frame as usize];
            scale.z = (*fp).u.kb.x8C.z * ftKb_Init_803CB710[frame as usize + 4];
            ext::HSD_JObjSetScale(jobj, scale);
            sn!(fp).x8 += 1;
        } else {
            ext::HSD_JObjSetScale(jobj, &(*fp).u.kb.x8C);
        }
    }
}

/// `ftKb_AirScaleAnimStep` (static inline): the same table read through `scale_base[frame + 4]`.
pub fn ftKb_AirScaleAnimStep(gobj: G, scale: &mut Vec3) {
    ftKb_PrScaleAnimStep(gobj, scale)
}

/// `ftKb_PrRestoreScale` (static).
pub fn ftKb_PrRestoreScale(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let jobj = jobj_of(gobj);
        ext::HSD_JObjSetScale(jobj, &(*fp).u.kb.x8C);
        rot_y(fp, FtPart_TopN as i32, M_PI_2 * (*fp).facing_dir as f64);
        if sn!(fp).facing_dir != 0.0 {
            (*fp).facing_dir = sn!(fp).facing_dir;
        }
        sn!(fp).facing_dir = 0.0;
    }
}

/// `ftKb_PrSetCallbacks` (static inline).
pub fn ftKb_PrSetCallbacks(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        ftKb_SpecialN_set_cbs(gobj);
        (*fp).deal_dmg_cb = Some(cb_fn_80100E0C);
        (*fp).x21F8 = Some(cb_fn_80105978);
    }
}

/// `ftKb_PrHitCapsuleToggle` (static inline): flip hit capsule `x4` every `unk6` frames.
pub fn ftKb_PrHitCapsuleToggle(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        sn!(fp).xC += 1;
        if sn!(fp).xC >= da.specialn_pr_unk6 as i32 && (*fp).x914[0].state != HitCapsule_Disabled {
            (*fp).x914[0].x4 = ((*fp).x914[0].x4 + 1) & 1;
            sn!(fp).xC = 0;
        }
    }
}

/// `ftKb_PrPlayRollSFX` (static inline).
pub fn ftKb_PrPlayRollSFX(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let sfx_id = if (*fp).ground_or_air == GA_Air { 0x3D0D3 } else { 0x3D0D0 };
        ext::ft_80088510(fp, sfx_id, 127, 64);
    }
}

/// `ftKb_PrWallBounceEffect` (static): dust + quake + sfx at the wall.
pub fn ftKb_PrWallBounceEffect(gobj: G, fp: *mut Fighter, dir: f32, pos: &mut Vec3, angle: &mut f32) {
    unsafe {
        let fp2 = fpp(gobj);
        *pos = (*fp2).cur_pos;
        *angle = atan2f(
            -(*fp).coll_data.left_facing_wall.normal.x * dir,
            (*fp).coll_data.left_facing_wall.normal.y,
        );
        if dir > 0.0 {
            pos.x += abs_f((*fp2).coll_data.ecb.right.x);
        } else {
            pos.x -= abs_f((*fp2).coll_data.ecb.left.x);
        }
        pos.y = fmadds(0.5, abs_f((*fp2).coll_data.ecb.top.y + (*fp2).coll_data.ecb.bottom.y), pos.y);
        ext::efSync_Spawn(0x406, gobj, &[pos as *mut Vec3 as *mut core::ffi::c_void, angle as *mut f32 as *mut core::ffi::c_void]);
        ext::Camera_RequestQuake(QuakeKind_Medium, pos);
        st::ftCommon_8007EBAC(&mut *fp2, 0xC, 0xA);
        fh::play_sfx(&mut *fp2, 0x3D0D6, 127, 64);
    }
}

// ---------------------------------------------------------------------------------------------
// Callbacks and helpers
// ---------------------------------------------------------------------------------------------

/// `fn_80100E0C`: deal-damage callback; bounce back with the Hit state.
pub fn fn_80100E0C(gobj: G) {
    const MF: u32 = (1 << 1) | (1 << 4) | (1 << 7) | (1 << 9) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27);
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        sn!(fp).xC = 0;
        let m = (*fp).motion_id as i32;
        if m == ftKb_MS_PrSpecialN1 as i32
            || m == ftKb_MS_PrSpecialNTurn as i32
            || m == ftKb_MS_PrSpecialAirN as i32
            || m == ftKb_MS_PrSpecialN0 as i32
        {
            sn!(fp).x0 = (sn!(fp).x0 as u32).wrapping_sub(da.specialn_pr_unk) as i32;
            (*fp).facing_dir = sn!(fp).x34.x;
            ext::ftColl_8007AFF8(gobj);
            st::Fighter_ChangeMotionState(gobj, ftKb_MS_PrSpecialNHit as i32, MF, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
            ftKb_SpecialN_set_cbs(gobj);
            (*fp).deal_dmg_cb = None;
            if (*fp).ground_or_air == GA_Ground {
                (*fp).self_vel.x = (*fp).gr_vel * da.specialn_pr_horizontal_bounce_momentum_on_hit;
                ext::ftCommon_8007D5D4(fp);
            } else {
                (*fp).self_vel.x *= da.specialn_pr_horizontal_bounce_momentum_on_hit;
            }
            (*fp).self_vel.y = da.specialn_pr_vertical_bounce_momentum_on_hit;
            (*fp).x74_self_accel.y = 0.0;
            (*fp).x74_self_accel.x = 0.0;
            (*fp).xE4_ground_accel_1 = 0.0;
            (*fp).gr_vel = 0.0;
            sn!(fp).facing_dir = 0.0;
            rot_y(fp, FtPart_TopN as i32, M_PI_2);
            fh::play_sfx(&mut *fp, 0x3D0D6, 127, 64);
        }
    }
}

pub fn fn_80100F60(gobj: G) {
    ext::ft_80089824(gobj);
    ext::ft_800892A0(gobj);
}

/// Rollout hit capsule: disable below the speed threshold, otherwise scale damage with speed.
pub fn ftKb_SpecialNPr_80100F94(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        let state = (*fp).x914[0].state;
        let abs_vel = if (*fp).ground_or_air == GA_Air { abs_f((*fp).self_vel.x) } else { abs_f((*fp).gr_vel) };
        if abs_vel < da.specialn_pr_unk10 {
            (*fp).x914[0].state = HitCapsule_Disabled;
            ext::ftCo_800BFFAC(fp);
        } else if state == HitCapsule_Disabled {
            (*fp).x914[0].state = HitCapsule_Enabled;
        }
        if (*fp).x914[0].state != HitCapsule_Disabled {
            let mut damage: i32 = if (*fp).ground_or_air == GA_Air {
                (da.specialn_pr_damage_multiplier * (da.specialn_pr_base_damage + abs_f((*fp).self_vel.x))) as i32
            } else {
                (da.specialn_pr_damage_multiplier * (da.specialn_pr_base_damage + abs_f((*fp).gr_vel))) as i32
            };
            if damage < 1 {
                damage = 1;
            }
            ext::ftColl_8007ABD0(&mut (*fp).x914[0], damage as u32, gobj);
        }
    }
}

/// End of the roll: switch to the End state (ground when `!unk`, air when `unk`).
pub fn ftKb_SpecialNPr_801010D4(gobj: G, unk: bool, flags: u32, anim_start: f32) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        (*fp).facing_dir = sn!(fp).x34.x;
        ext::ftColl_8007AFF8(gobj);
        if !unk {
            let msid = if (*fp).facing_dir == 1.0 { ftKb_MS_PrSpecialNEndR } else { ftKb_MS_PrSpecialNEndL };
            st::Fighter_ChangeMotionState(gobj, msid as i32, flags, anim_start, 1.0, 0.0, core::ptr::null_mut());
            (*fp).gr_vel *= da.specialn_pr_forward_momentum_from_stick;
            (*fp).x74_self_accel.y = 0.0;
            (*fp).self_vel.y = 0.0;
        } else {
            let msid = if (*fp).facing_dir == 1.0 { ftKb_MS_PrSpecialAirNEndR0 } else { ftKb_MS_PrSpecialAirNEndR1 };
            st::Fighter_ChangeMotionState(gobj, msid as i32, flags, anim_start, 1.0, 0.0, core::ptr::null_mut());
            (*fp).self_vel.x *= da.specialn_pr_forward_momentum_from_stick;
            (*fp).self_vel.y *= da.specialn_pr_unk4;
            (*fp).xE4_ground_accel_1 = 0.0;
            (*fp).gr_vel = 0.0;
        }
        ftKb_PrSetCallbacks(gobj);
        ftKb_PrRestoreScale(gobj);
        sn!(fp).facing_dir = 0.0;
        rot_y(fp, FtPart_TopN as i32, M_PI_2 * (*fp).facing_dir as f64);
    }
}

/// Initialise the roll state (shared by the ground and air entries).
pub fn ftKb_SpecialNPr_8010131C(gobj: G) {
    unsafe {
        let jobj = jobj_of(gobj);
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        ext::HSD_JObjGetScale(jobj, &mut (*fp).u.kb.x8C);
        (*fp).gr_vel = 0.0;
        sn!(fp).x0 = da.specialn_pr_duration as i32;
        sn!(fp).x4 = -1;
        sn!(fp).x8 = -1;
        sn!(fp).xC = 0;
        sn!(fp).x14 = 0.0;
        sn!(fp).x34.y = 0.0;
        sn!(fp).facing_dir = 0.0;
        sn!(fp).x24 = 0;
        sn!(fp).x28 = 0;
        sn!(fp).x2C = da.specialn_pr_charge_rate1 as i32;
        sn!(fp).x30 = 0;
        (*fp).deal_dmg_cb = Some(cb_fn_80100E0C);
        (*fp).x21F8 = Some(cb_fn_80105978);
    }
}

/// Wrap the roll angle and play the roll sfx when it crosses the turn point.
pub fn ftKb_SpecialNPr_8010140C(gobj: G, arg1: bool) {
    unsafe {
        let fp = fpp(gobj);
        while sn!(fp).x14 < 0.0 {
            sn!(fp).x14 = (sn!(fp).x14 as f64 + M_PI * 2.0) as f32;
        }
        while sn!(fp).x14 as f64 > M_PI * 2.0 {
            sn!(fp).x14 = (sn!(fp).x14 as f64 - M_PI * 2.0) as f32;
        }
        if sn!(fp).x34.y == 0.0 {
            ftKb_PrPlayRollSFX(gobj);
        } else if sn!(fp).x34.x == (if arg1 { -1.0 } else { 1.0 }) {
            if sn!(fp).x14 < sn!(fp).x34.y {
                ftKb_PrPlayRollSFX(gobj);
            }
        } else if sn!(fp).x14 > sn!(fp).x34.y {
            ftKb_PrPlayRollSFX(gobj);
        }
        sn!(fp).x34.y = sn!(fp).x14;
    }
}

/// Ground Rollout entry.
pub fn ftKb_SpecialNPr_80101560(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        sn!(fp).x34.x = (*fp).facing_dir;
        let msid = if 1.0 == sn!(fp).x34.x { ftKb_MS_PrSpecialNStartR } else { ftKb_MS_PrSpecialNStartL };
        st::Fighter_ChangeMotionState(gobj, msid as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::Fighter_ClearCmdVars(&mut *fp);
        st::ftAnim_8006EBA4(gobj);
        ftKb_SpecialNPr_8010131C(gobj);
        (*fp).x74_self_accel.y = 0.0;
        (*fp).self_vel.y = 0.0;
        sn!(fp).x1C = da.specialn_pr_unk1;
    }
}

/// Air Rollout entry.
pub fn ftKb_SpecialNPr_80101618(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        sn!(fp).x34.x = (*fp).facing_dir;
        let msid = if 1.0 == sn!(fp).x34.x { ftKb_MS_PrSpecialAirNStartR } else { ftKb_MS_PrSpecialAirNStartL };
        st::Fighter_ChangeMotionState(gobj, msid as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::Fighter_ClearCmdVars(&mut *fp);
        st::ftAnim_8006EBA4(gobj);
        ftKb_SpecialNPr_8010131C(gobj);
        (*fp).x74_self_accel.y = da.specialn_pr_air_height_offset_at_start;
        sn!(fp).x1C = da.specialn_pr_unk2;
    }
}

// ---------------------------------------------------------------------------------------------
// Anim
// ---------------------------------------------------------------------------------------------

const MF_START_LOOP: u32 = (1 << 1) | (1 << 4) | (1 << 18);
const MF_LOOP_FULL: u32 = (1 << 1) | (1 << 4) | (1 << 7) | (1 << 9) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27);

pub fn ftKb_PrSpecialNStart_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        sn!(fp).facing_dir = 0.0;
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::Fighter_ChangeMotionState(gobj, ftKb_MS_PrSpecialNLoop as i32, MF_START_LOOP, 0.0, 0.0, 0.0, core::ptr::null_mut());
            ftKb_PrSetCallbacks(gobj);
            (*fp).cur_anim_frame = 0.0;
            st::ftAnim_SetAnimRate(gobj, 0.0);
            let v = (*fp).facing_dir * 0.0001;
            (*fp).self_vel.x = v;
            (*fp).gr_vel = v;
            (*fp).x74_self_accel.x = 0.0;
            (*fp).xE4_ground_accel_1 = 0.0;
            rot_y(fp, 0, M_PI_2);
        }
    }
}

pub fn ftKb_PrSpecialAirNStart_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        sn!(fp).facing_dir = 0.0;
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::Fighter_ChangeMotionState(gobj, ftKb_MS_PrSpecialAirNLoop as i32, MF_START_LOOP, 0.0, 0.0, 0.0, core::ptr::null_mut());
            ftKb_PrSetCallbacks(gobj);
            (*fp).cur_anim_frame = 0.0;
            st::ftAnim_SetAnimRate(gobj, 0.0);
            (*fp).self_vel.x = (*fp).facing_dir * 0.0001;
            (*fp).x74_self_accel.x = 0.0;
            rot_y(fp, 0, M_PI_2);
        }
    }
}

/// The charge-spin step: `x14 += x34.x * (x2C * MTXDegToRad(spin))`, a fused f32 site.
unsafe fn charge_spin(fp: *mut Fighter, da: &ftKb_DatAttrs) {
    let prod = (sn!(fp).x2C as f32) * MTXDegToRad(da.specialn_pr_charge_spin_animation);
    sn!(fp).x14 = fmadds(sn!(fp).x34.x, prod, sn!(fp).x14);
}

/// Charge step: `x2C += rate2` (an int += float), clamp at the charge time.
unsafe fn charge_add(fp: *mut Fighter, da: &ftKb_DatAttrs) -> bool {
    sn!(fp).x2C = (sn!(fp).x2C as f32 + da.specialn_pr_charge_rate2) as i32;
    if sn!(fp).x2C as f32 >= da.specialn_pr_charge_time {
        sn!(fp).x2C = da.specialn_pr_charge_time as i32;
        return true;
    }
    false
}

unsafe fn charge_loop_anim(gobj: G, full_ms: i32) {
    let fp = fpp(gobj);
    let da = &*da_of(fp);
    ftKb_SpecialNPr_8010140C(gobj, false);
    sn!(fp).facing_dir = 0.0;
    if charge_add(fp, da) {
        if sn!(fp).x30 == 0 {
            ext::ftCo_800BFFD0(fp, 5, 0);
        }
        sn!(fp).x30 = 1;
        st::Fighter_ChangeMotionState(gobj, full_ms, MF_LOOP_FULL, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
    }
    charge_spin(fp, da);
    ftPr_NormalizeAndSetRollAngle(gobj);
    rot_y(fp, FtPart_TopN as i32, M_PI_2);
}

unsafe fn charge_full_anim(gobj: G) {
    let fp = fpp(gobj);
    let da = &*da_of(fp);
    ftKb_SpecialNPr_8010140C(gobj, false);
    sn!(fp).facing_dir = 0.0;
    if charge_add(fp, da) {
        sn!(fp).x30 = 1;
    }
    charge_spin(fp, da);
    ftPr_NormalizeAndSetRollAngle(gobj);
    rot_y(fp, FtPart_TopN as i32, M_PI_2);
}

pub fn ftKb_PrSpecialNLoop_Anim(gobj: G) {
    unsafe { charge_loop_anim(gobj, ftKb_MS_PrSpecialNFull as i32) }
}

pub fn ftKb_PrSpecialNFull_Anim(gobj: G) {
    unsafe { charge_full_anim(gobj) }
}

pub fn ftKb_PrSpecialAirNLoop_Anim(gobj: G) {
    unsafe { charge_loop_anim(gobj, ftKb_MS_PrSpecialAirNFull as i32) }
}

pub fn ftKb_PrSpecialAirNFull_Anim(gobj: G) {
    unsafe { charge_full_anim(gobj) }
}

/// Shared tail of `N1_Anim` / `AirN_Anim`: wrap-around roll end checks. Returns true when it left
/// for the End state.
unsafe fn roll_end_check(gobj: G, fp: *mut Fighter, old_angle: f32, delta: f32, air: bool, air_rot_direct: bool) {
    ftPr_NormalizeAndSetRollAngle(gobj);
    sn!(fp).x0 -= 1;
    let x14 = sn!(fp).x14 as f64;
    let set_rot = |fp: *mut Fighter| {
        if air_rot_direct {
            ext::HSD_JObjSetRotationY((*(*fp).parts.add(FtPart_TopN as usize)).joint as *mut _, M_PI_2 as f32);
        } else {
            rot_y(fp, FtPart_TopN as i32, M_PI_2);
        }
    };
    if sn!(fp).x0 <= 0 {
        if M_PI_2 < x14 && x14 < M_PI_2 * 3.0 {
            if delta > 0.0 {
                if x14 > M_PI && (old_angle as f64) < M_PI {
                    sn!(fp).x0 = 0;
                    ftKb_SpecialNPr_801010D4(gobj, air, 0x40012, 0.0);
                    return;
                }
                set_rot(fp);
                return;
            }
            if x14 < M_PI && old_angle as f64 > M_PI {
                sn!(fp).x0 = 0;
                ftKb_SpecialNPr_801010D4(gobj, air, 0x40012, 0.0);
                return;
            }
            set_rot(fp);
            return;
        }
        set_rot(fp);
        return;
    }
    rot_y(fp, FtPart_TopN as i32, M_PI_2);
}

pub fn ftKb_PrSpecialN1_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftKb_SpecialNPr_8010140C(gobj, false);
        sn!(fp).facing_dir = 0.0;
        ftKb_PrScaleAnimStep(gobj, &mut scale);
        ftKb_PrHitCapsuleToggle(gobj);
        ftKb_SpecialNPr_80100F94(gobj);
        let old_angle = sn!(fp).x14;
        let delta = MTXDegToRad(sn!(fp).x2C as f32) * ((0.2f64 * da.specialn_pr_unk5 as f64 * sn!(fp).x34.x as f64) as f32);
        sn!(fp).x14 = old_angle + delta;
        roll_end_check(gobj, fp, old_angle, delta, false, false);
    }
}

pub fn ftKb_PrSpecialAirN_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftKb_SpecialNPr_8010140C(gobj, false);
        sn!(fp).facing_dir = 0.0;
        ftKb_AirScaleAnimStep(gobj, &mut scale);
        ftKb_PrHitCapsuleToggle(gobj);
        ftKb_SpecialNPr_80100F94(gobj);
        let old_angle = sn!(fp).x14;
        let delta = ((0.2f64 * da.specialn_pr_unk5 as f64 * sn!(fp).x34.x as f64) as f32)
            * (0.017453292f32 * sn!(fp).x2C as f32 * da.specialn_pr_spin_anim_speed_after_collision);
        sn!(fp).x14 = old_angle + delta;
        roll_end_check(gobj, fp, old_angle, delta, true, true);
    }
}

pub fn ftKb_PrSpecialNTurn_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftKb_SpecialNPr_8010140C(gobj, true);
        ftKb_PrScaleAnimStep(gobj, &mut scale);
        // f64 fused site: x14 = fma(0.2 * turn_speed, -x34.x, x14)
        let a = 0.2f64 * da.specialn_pr_spinning_speed_turn as f64;
        let b = (-sn!(fp).x34.x) as f64;
        sn!(fp).x14 = a.mul_add(b, sn!(fp).x14 as f64) as f32;
        ftPr_NormalizeAndSetRollAngle(gobj);
        sn!(fp).x0 -= 1;
        if sn!(fp).x0 <= 0 {
            sn!(fp).x0 = 0;
            sn!(fp).x34.x = -sn!(fp).x34.x;
            ftKb_SpecialNPr_801010D4(gobj, false, 0x40012, 0.0);
            return;
        }
        if sn!(fp).x24 % (da.specialn_pr_delay_per_smoke as i32) == 0 {
            let mut angle = atan2f(-(*fp).coll_data.floor.normal.x, (*fp).coll_data.floor.normal.y);
            ext::efSync_Spawn(
                0x3FF,
                gobj,
                &[
                    &mut (*fp).cur_pos as *mut Vec3 as *mut core::ffi::c_void,
                    &mut (*fp).facing_dir as *mut f32 as *mut core::ffi::c_void,
                    &mut angle as *mut f32 as *mut core::ffi::c_void,
                ],
            );
        }
        sn!(fp).x24 += 1;
        rot_y(fp, FtPart_TopN as i32, M_PI_2);
    }
}

pub fn ftKb_PrSpecialNEnd_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        sn!(fp).facing_dir = 0.0;
        ftKb_PrScaleAnimStep(gobj, &mut scale);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ftKb_PrRestoreScale(gobj);
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftKb_PrSpecialN_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftKb_SpecialNPr_8010140C(gobj, true);
        ftKb_PrScaleAnimStep(gobj, &mut scale);
        let v = (0.2f64 * da.specialn_pr_spinning_speed_turn as f64 * (-sn!(fp).x34.x) as f64) as f32;
        sn!(fp).x14 = fmadds(da.specialn_pr_spin_anim_speed_after_collision, v, sn!(fp).x14);
        ftPr_NormalizeAndSetRollAngle(gobj);
        sn!(fp).x0 -= 1;
        if sn!(fp).x0 <= 0 {
            sn!(fp).x0 = 0;
            sn!(fp).x34.x = -sn!(fp).x34.x;
            ftKb_SpecialNPr_801010D4(gobj, false, 0x40012, 0.0);
            return;
        }
        sn!(fp).x24 += 1;
        rot_y(fp, FtPart_TopN as i32, M_PI_2);
    }
}

pub fn ftKb_PrSpecialAirNEndR_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        sn!(fp).facing_dir = 0.0;
        ftKb_PrScaleAnimStep(gobj, &mut scale);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ftKb_PrRestoreScale(gobj);
            if da.specialn_pr_freefall_toggle == 0.0 {
                st::ftCo_Fall_Enter(gobj);
            } else {
                st::ftCo_80096900(gobj, 1, 0, 1, 1.0, da.specialn_pr_freefall_toggle);
            }
        }
    }
}

pub fn ftKb_PrSpecialNHit_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftKb_PrScaleAnimStep(gobj, &mut scale);
        let v = (0.2f64 * da.specialn_pr_spinning_speed_turn as f64 * (-sn!(fp).x34.x) as f64) as f32;
        sn!(fp).x14 = fmadds(da.specialn_pr_spin_anim_speed_after_collision, v, sn!(fp).x14);
        ftPr_NormalizeAndSetRollAngle(gobj);
        rot_y(fp, FtPart_TopN as i32, M_PI_2);
    }
}

// ---------------------------------------------------------------------------------------------
// IASA
// ---------------------------------------------------------------------------------------------

pub fn ftKb_PrSpecialNStart_IASA(_gobj: G) {}
pub fn ftKb_PrSpecialNTurn_IASA(_gobj: G) {}
pub fn ftKb_PrSpecialNEnd_IASA(_gobj: G) {}
pub fn ftKb_PrSpecialAirNStart_IASA(_gobj: G) {}
pub fn ftKb_PrSpecialAirN_IASA(_gobj: G) {}
pub fn ftKb_PrSpecialN_IASA(_gobj: G) {}
pub fn ftKb_PrSpecialAirNEndR_IASA(_gobj: G) {}
pub fn ftKb_PrSpecialNHit_IASA(_gobj: G) {}

const MF_RELEASE: u32 = (1 << 1) | (1 << 4) | (1 << 7) | (1 << 12) | (1 << 18);

/// Release B: launch the roll on the ground.
unsafe fn release_ground(gobj: G) {
    let fp = fpp(gobj);
    let da = &*da_of(fp);
    if ((*fp).input.held_buttons[0] & HSD_PAD_B as u32) == 0 {
        st::Fighter_ChangeMotionState(gobj, ftKb_MS_PrSpecialN1 as i32, MF_RELEASE, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
        ftKb_PrSetCallbacks(gobj);
        st::ftAnim_8006EBA4(gobj);
        (*fp).gr_vel = sn!(fp).x34.x * ((sn!(fp).x2C as f32 - da.specialn_pr_charge_rate1) * da.specialn_pr_air_speed);
        rot_y(fp, FtPart_TopN as i32, M_PI_2);
        ftPr_NormalizeAndSetRollAngle(gobj);
        fh::play_sfx(&mut *fp, 0x3D0D9, 127, 64);
    }
}

/// Release B: launch the roll in the air.
unsafe fn release_air(gobj: G) {
    let fp = fpp(gobj);
    let da = &*da_of(fp);
    if ((*fp).input.held_buttons[0] & HSD_PAD_B as u32) == 0 {
        st::Fighter_ChangeMotionState(gobj, ftKb_MS_PrSpecialAirN as i32, MF_RELEASE, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
        ftKb_PrSetCallbacks(gobj);
        st::ftAnim_8006EBA4(gobj);
        (*fp).self_vel.x = sn!(fp).x34.x * ((sn!(fp).x2C as f32 - da.specialn_pr_charge_rate1) * da.specialn_pr_air_speed);
        rot_y(fp, FtPart_TopN as i32, M_PI_2);
        ftPr_NormalizeAndSetRollAngle(gobj);
        fh::play_sfx(&mut *fp, 0x3D0D9, 127, 64);
    }
}

pub fn ftKb_PrSpecialNLoop_IASA(gobj: G) {
    unsafe { release_ground(gobj) }
}
pub fn ftKb_PrSpecialNFull_IASA(gobj: G) {
    unsafe { release_ground(gobj) }
}
pub fn ftKb_PrSpecialAirNLoop_IASA(gobj: G) {
    unsafe { release_air(gobj) }
}
pub fn ftKb_PrSpecialAirNFull_IASA(gobj: G) {
    unsafe { release_air(gobj) }
}

pub fn ftKb_PrSpecialN1_IASA(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        if abs_f((*fp).input.lstick[0].x) > da.specialn_pr_spinning_speed {
            let sign = signf((*fp).input.lstick[0].x);
            if sn!(fp).x34.x != sign {
                ext::ftColl_8007AFF8(gobj);
                sn!(fp).facing_dir = sign;
                sn!(fp).x34.x = -sign;
                st::Fighter_ChangeMotionState(gobj, ftKb_MS_PrSpecialNTurn as i32, 0x0C4C0292, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
                ftKb_PrSetCallbacks(gobj);
                sn!(fp).x10 = (*fp).gr_vel;
                sn!(fp).x1C = -0.05f32 * (*fp).gr_vel;
                sn!(fp).x24 = 0;
                ftPr_NormalizeAndSetRollAngle(gobj);
                fh::play_sfx(&mut *fp, 0x3D0D9, 127, 64);
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Phys
// ---------------------------------------------------------------------------------------------

// The root module's Fighter-only halves, with the C (gobj) signature.
pub fn ftKb_PrSpecialNStart_Phys(gobj: G) {
    unsafe { half::ftKb_PrSpecialNStart_Phys(&mut *fpp(gobj)) }
}
pub fn ftKb_PrSpecialNLoop_Phys(gobj: G) {
    ftKb_PrSpecialNStart_Phys(gobj);
}
pub fn ftKb_PrSpecialNFull_Phys(gobj: G) {
    ftKb_PrSpecialNStart_Phys(gobj);
}
pub fn ftKb_PrSpecialNEnd_Phys(gobj: G) {
    unsafe { half::ftKb_PrSpecialNEnd_Phys(&mut *fpp(gobj)) }
}
pub fn ftKb_PrSpecialAirNStart_Phys(gobj: G) {
    unsafe { half::ftKb_PrSpecialAirNStart_Phys(&mut *fpp(gobj)) }
}
pub fn ftKb_PrSpecialAirNLoop_Phys(gobj: G) {
    unsafe { half::ftKb_PrSpecialAirNLoop_Phys(&mut *fpp(gobj)) }
}
pub fn ftKb_PrSpecialAirNFull_Phys(gobj: G) {
    unsafe { half::ftKb_PrSpecialAirNFull_Phys(&mut *fpp(gobj)) }
}
pub fn ftKb_PrSpecialAirNEndR_Phys(gobj: G) {
    unsafe { half::ftKb_PrSpecialAirNEndR_Phys(&mut *fpp(gobj)) }
}
pub fn ftKb_PrSpecialNHit_Phys(gobj: G) {
    unsafe { half::ftKb_PrSpecialNHit_Phys(&mut *fpp(gobj)) }
}

pub fn ftKb_PrSpecialN1_Phys(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        let slope = (*fp).coll_data.floor.normal.x;
        let base_vel = sn!(fp).x34.x * (da.specialn_pr_air_speed * (sn!(fp).x2C as f32 - da.specialn_pr_some_speed_var));
        let influence = da.specialn_pr_unk9 * (base_vel * abs_f(slope));
        if (*fp).coll_data.floor.normal.x > 0.0 {
            // both arms of the C's `x1C > 0` test are identical
            (*fp).gr_vel = base_vel + influence;
        } else {
            (*fp).gr_vel = base_vel - influence;
        }
        let mut max_vel = da.specialn_pr_base_speed;
        if abs_f((*fp).gr_vel) > max_vel {
            if (*fp).gr_vel < 0.0 {
                max_vel = -max_vel;
            }
            (*fp).gr_vel = max_vel;
        }
        let mut max_vel = da.specialn_pr_max_speed;
        if abs_f((*fp).gr_vel) > max_vel {
            if (*fp).gr_vel < 0.0 {
                max_vel = -max_vel;
            }
            (*fp).gr_vel = max_vel;
        }
        sn!(fp).x18 = abs_f((*fp).gr_vel);
        (*fp).x74_self_accel.y = 0.0;
        (*fp).self_vel.y = 0.0;
        ext::ftCommon_SetSelfMovementFromGroundedMovement(fp);
        sn!(fp).x2C = (sn!(fp).x2C as f32 - da.specialn_pr_unk8) as i32;
        if (sn!(fp).x2C as f32) < da.specialn_pr_some_speed_var {
            ftKb_SpecialNPr_801010D4(gobj, false, 0x40012, 0.0);
        }
    }
}

/// `gr_vel += scale * (x1C +- influence)` of `ftKb_PrSpecialNTurn_Phys` (a fused f32 site in every arm).
pub fn turn_gr_vel(gr_vel: f32, scale: f32, x1c: f32, influence: f32, floor_nx_positive: bool) -> f32 {
    if floor_nx_positive {
        fmadds(scale, x1c + influence, gr_vel)
    } else {
        fmadds(scale, x1c - influence, gr_vel)
    }
}

pub fn ftKb_PrSpecialNTurn_Phys(gobj: G) {
    const MF1: u32 = (1 << 4) | (1 << 18) | (1 << 25);
    const MF2: u32 = (1 << 1) | (1 << 4) | (1 << 18) | (1 << 25);
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        let scale = da.specialn_pr_turn_rate_related * ext::mpLib_800569EC((*fp).coll_data.floor.flags);
        let slope = abs_f((*fp).coll_data.floor.normal.x);
        let x1c = sn!(fp).x1C;
        let influence = da.specialn_pr_unk9 * (x1c * slope);
        (*fp).gr_vel = turn_gr_vel((*fp).gr_vel, scale, x1c, influence, (*fp).coll_data.floor.normal.x > 0.0);
        (*fp).x74_self_accel.y = 0.0;
        (*fp).self_vel.y = 0.0;
        ext::ftCommon_SetSelfMovementFromGroundedMovement(fp);
        if sn!(fp).x10 > 0.0 {
            if (*fp).gr_vel < 0.0 && abs_f((*fp).gr_vel) >= abs_f(sn!(fp).x10 * da.specialn_pr_unk11) {
                (*fp).x21EC = Some(cb_fn_80100F60);
                st::Fighter_ChangeMotionState(gobj, ftKb_MS_PrSpecialN1 as i32, MF1, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
                ftKb_PrSetCallbacks(gobj);
                ftPr_SetFacingDir(fp);
            }
        } else if (*fp).gr_vel > 0.0 && abs_f((*fp).gr_vel) >= abs_f(sn!(fp).x10 * da.specialn_pr_unk11) {
            (*fp).x21EC = Some(cb_fn_80100F60);
            st::Fighter_ChangeMotionState(gobj, ftKb_MS_PrSpecialN1 as i32, MF2, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
            ftKb_PrSetCallbacks(gobj);
            ftPr_SetFacingDir(fp);
        }
        rot_y(fp, FtPart_TopN as i32, M_PI_2);
        ftPr_NormalizeAndSetRollAngle(gobj);
    }
}

/// Air-roll horizontal speed shaping shared by `AirN_Phys` and `N_Phys`: subtract the decel, floor the
/// magnitude at `min_vel`.
pub fn air_decel_floor(vel_x: f32, decel_attr: f32, min_attr: f32) -> f32 {
    let decel = if vel_x > 0.0 { decel_attr } else { -decel_attr };
    let mut vx = vel_x - decel;
    let abs_vel = abs_f(vx);
    let mut min_vel = min_attr;
    if abs_vel < min_vel {
        if vx < 0.0 {
            min_vel = -min_vel;
        }
        vx = min_vel;
    }
    vx
}

pub fn ftKb_PrSpecialAirN_Phys(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        (*fp).self_vel.x = sn!(fp).x34.x * (da.specialn_pr_air_speed * (sn!(fp).x2C as f32 - da.specialn_pr_some_speed_var));
        (*fp).self_vel.x = air_decel_floor((*fp).self_vel.x, da.specialn_pr_air_x_axis_momentum, da.specialn_pr_air_y_axis_momentum);
        sn!(fp).x18 = abs_f((*fp).self_vel.x);
        ext::ftCommon_Fall(fp, da.specialn_pr_air_height_offset_at_start, da.specialn_pr_bounciness);
        sn!(fp).x2C = (sn!(fp).x2C as f32 - da.specialn_pr_unk8) as i32;
        if (sn!(fp).x2C as f32) < da.specialn_pr_some_speed_var {
            ftKb_SpecialNPr_801010D4(gobj, true, 0x40012, 0.0);
        }
    }
}

pub fn ftKb_PrSpecialN_Phys(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        (*fp).self_vel.x = air_decel_floor((*fp).self_vel.x, da.specialn_pr_air_x_axis_momentum, da.specialn_pr_air_y_axis_momentum);
        ext::ftCommon_Fall(fp, da.specialn_pr_air_height_offset_at_start, da.specialn_pr_bounciness);
        rot_y(fp, FtPart_TopN as i32, M_PI_2);
        ftPr_NormalizeAndSetRollAngle(gobj);
    }
}

// ---------------------------------------------------------------------------------------------
// Coll
// ---------------------------------------------------------------------------------------------

pub fn ftKb_PrSpecialNStart_Coll(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            ext::ftCommon_8007D5D4(fp);
            let msid = if 1.0 == sn!(fp).x34.x { ftKb_MS_PrSpecialAirNStartR } else { ftKb_MS_PrSpecialAirNStartL };
            st::Fighter_ChangeMotionState(gobj, msid as i32, 0x0C4C5092, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
            ftKb_PrSetCallbacks(gobj);
        }
    }
}

/// Ground Loop/Full -> the air version of the same state (shared tail).
unsafe fn to_air_charge(gobj: G, ms: i32) {
    let fp = fpp(gobj);
    ext::ftCommon_8007D5D4(fp);
    st::Fighter_ChangeMotionState(gobj, ms, 0x41092, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
    ftKb_PrSetCallbacks(gobj);
    ftPr_NormalizeAndSetRollAngle(gobj);
    rot_y(fp, FtPart_TopN as i32, M_PI_2);
}

/// Air Loop/Full -> the ground version of the same state (shared tail).
unsafe fn to_ground_charge(gobj: G, ms: i32) {
    let fp = fpp(gobj);
    ext::ftCommon_8007D7FC(fp);
    st::Fighter_ChangeMotionState(gobj, ms, 0x41092, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
    ftKb_PrSetCallbacks(gobj);
    ftPr_NormalizeAndSetRollAngle(gobj);
    rot_y(fp, FtPart_TopN as i32, M_PI_2);
}

pub fn ftKb_PrSpecialNLoop_Coll(gobj: G) {
    unsafe {
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            to_air_charge(gobj, ftKb_MS_PrSpecialAirNLoop as i32);
        }
    }
}

pub fn ftKb_PrSpecialNFull_Coll(gobj: G) {
    unsafe {
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            to_air_charge(gobj, ftKb_MS_PrSpecialAirNFull as i32);
        }
    }
}

/// Wall bounce shared by `N1_Coll` / `AirN_Coll`; returns whether a wall was hit.
unsafe fn wall_check(gobj: G, fp: *mut Fighter) -> bool {
    let wall_hit;
    if sn!(fp).x34.x == 1.0 {
        wall_hit = ((*fp).coll_data.env_flags & 0x3F) != 0;
        if wall_hit {
            let (mut pos, mut angle) = (Vec3 { x: 0.0, y: 0.0, z: 0.0 }, 0.0f32);
            ftKb_PrWallBounceEffect(gobj, fp, 1.0, &mut pos, &mut angle);
        }
    } else {
        wall_hit = ((*fp).coll_data.env_flags & 0xFC0) != 0;
        if wall_hit {
            let (mut pos, mut angle) = (Vec3 { x: 0.0, y: 0.0, z: 0.0 }, 0.0f32);
            ftKb_PrWallBounceEffect(gobj, fp, -1.0, &mut pos, &mut angle);
        }
    }
    wall_hit
}

pub fn ftKb_PrSpecialN1_Coll(gobj: G) {
    const MF: u32 = (1 << 1) | (1 << 3) | (1 << 4) | (1 << 7) | (1 << 12) | (1 << 14) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27);
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        let coll = ext::ft_80082888(gobj, &raw mut ftKb_Init_803CB758);
        let wall_hit = wall_check(gobj, fp);
        if wall_hit {
            sn!(fp).x2C = (sn!(fp).x2C as f32 * da.specialn_pr_unk12) as i32;
            if sn!(fp).x2C < 0 {
                sn!(fp).x2C = 0;
            }
            sn!(fp).x18 *= da.specialn_pr_unk12;
            (*fp).gr_vel = -(*fp).gr_vel * da.specialn_pr_unk12;
            (*fp).self_vel.x = -(*fp).self_vel.x * da.specialn_pr_unk12;
            sn!(fp).x34.x = if (*fp).gr_vel > 0.0 { 1.0 } else { -1.0 };
            rot_y(fp, FtPart_TopN as i32, M_PI_2);
        }
        if !coll {
            ext::ftCommon_8007D5D4(fp);
            st::Fighter_ChangeMotionState(gobj, ftKb_MS_PrSpecialAirN as i32, MF, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
            ftKb_PrSetCallbacks(gobj);
            ftPr_NormalizeAndSetRollAngle(gobj);
            rot_y(fp, FtPart_TopN as i32, M_PI_2);
            sn!(fp).x1C = da.specialn_pr_unk2;
        }
    }
}

pub fn ftKb_PrSpecialNTurn_Coll(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        let mut abs_vel = (*fp).gr_vel;
        if abs_vel < 0.0 {
            abs_vel = -abs_vel;
        }
        let result = if abs_vel > da.specialn_pr_unk3 {
            ext::ft_80082888(gobj, &raw mut ftKb_Init_803CB758)
        } else {
            ext::ft_80082978(gobj, &raw mut ftKb_Init_803CB758)
        };
        if !result {
            ext::ftCommon_8007D5D4(fp);
            st::Fighter_ChangeMotionState(gobj, ftKb_MS_PrSpecialN0 as i32, 0x0C4C5092, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
            ftKb_PrSetCallbacks(gobj);
        }
    }
}

pub fn ftKb_PrSpecialNEnd_Coll(gobj: G) {
    const MF: u32 = (1 << 1) | (1 << 4) | (1 << 7) | (1 << 12) | (1 << 14) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27);
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            ext::ftCommon_8007D5D4(fp);
            ftKb_SpecialNPr_801010D4(gobj, true, MF, (*fp).cur_anim_frame);
        }
    }
}

pub fn ftKb_PrSpecialAirNStart_Coll(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80081D0C(gobj) != GA_Ground as i32 {
            ext::ftCommon_8007D7FC(fp);
            let msid = if sn!(fp).x34.x == 1.0 { ftKb_MS_PrSpecialNStartR } else { ftKb_MS_PrSpecialNStartL };
            st::Fighter_ChangeMotionState(gobj, msid as i32, 0x0C4C5092, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
            ftKb_PrSetCallbacks(gobj);
        }
    }
}

pub fn ftKb_PrSpecialAirNLoop_Coll(gobj: G) {
    unsafe {
        if st::ft_80081D0C(gobj) != GA_Ground as i32 {
            to_ground_charge(gobj, ftKb_MS_PrSpecialNLoop as i32);
        }
    }
}

pub fn ftKb_PrSpecialAirNFull_Coll(gobj: G) {
    unsafe {
        if st::ft_80081D0C(gobj) != GA_Ground as i32 {
            to_ground_charge(gobj, ftKb_MS_PrSpecialNFull as i32);
        }
    }
}

pub fn ftKb_PrSpecialAirN_Coll(gobj: G) {
    const MF: u32 = (1 << 1) | (1 << 3) | (1 << 4) | (1 << 7) | (1 << 12) | (1 << 14) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27);
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        let coll = if sn!(fp).x34.x == 1.0 {
            ext::ft_8008239C(gobj, 1, &raw mut ftKb_Init_803CB758)
        } else {
            ext::ft_8008239C(gobj, -1, &raw mut ftKb_Init_803CB758)
        };
        let wall_hit = wall_check(gobj, fp);
        if wall_hit {
            sn!(fp).x2C = (sn!(fp).x2C as f32 * da.specialn_pr_unk12) as i32;
            if sn!(fp).x2C < 0 {
                sn!(fp).x2C = 0;
            }
            sn!(fp).x18 *= da.specialn_pr_unk12;
            (*fp).self_vel.x = -(*fp).self_vel.x * da.specialn_pr_unk12;
            sn!(fp).x34.x = -sn!(fp).x34.x;
        }
        if coll {
            let vel_y = abs_f((*fp).self_vel.y * da.specialn_pr_bounce1);
            (*fp).self_vel.y = vel_y;
            if vel_y < da.specialn_pr_bounce2 {
                ext::ftCommon_8007D7FC(fp);
                st::Fighter_ChangeMotionState(gobj, ftKb_MS_PrSpecialN1 as i32, MF, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
                ftKb_PrSetCallbacks(gobj);
                let vel = sn!(fp).x18 * sn!(fp).x34.x;
                (*fp).gr_vel = vel;
                (*fp).self_vel.x = vel;
                (*fp).self_vel.z = 0.0;
                (*fp).self_vel.y = 0.0;
                ftPr_NormalizeAndSetRollAngle(gobj);
                rot_y(fp, FtPart_TopN as i32, M_PI_2);
                ext::efSync_Spawn(
                    0x3FF,
                    gobj,
                    &[
                        &mut (*fp).cur_pos as *mut Vec3 as *mut core::ffi::c_void,
                        &mut sn!(fp).x34 as *mut Vec3 as *mut core::ffi::c_void,
                        &mut sn!(fp).x14 as *mut f32 as *mut core::ffi::c_void,
                    ],
                );
                sn!(fp).x1C = da.specialn_pr_unk1;
            } else {
                if abs_f((*fp).input.lstick[0].x) > da.specialn_pr_spinning_speed {
                    let dir = signf((*fp).input.lstick[0].x);
                    sn!(fp).x34.x = dir;
                    let v = sn!(fp).x18 * sn!(fp).x34.x;
                    (*fp).gr_vel = v;
                    (*fp).self_vel.x = v;
                    rot_y(fp, FtPart_TopN as i32, M_PI_2);
                }
                ftPr_NormalizeAndSetRollAngle(gobj);
            }
            sn!(fp).x8 = 0;
            return;
        }
        if st::ftCliffCommon_80081298(gobj) != 0 {
            ftKb_PrRestoreScale(gobj);
            ext::ftCliffCommon_80081370(gobj);
        }
    }
}

pub fn ftKb_PrSpecialN_Coll(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if ext::ft_800824A0(gobj, &raw mut ftKb_Init_803CB758) {
            ext::ftCommon_8007D7FC(fp);
            st::Fighter_ChangeMotionState(gobj, ftKb_MS_PrSpecialNTurn as i32, 0x0C4C5092, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
            ftKb_PrSetCallbacks(gobj);
            (*fp).gr_vel = (*fp).self_vel.x;
            (*fp).self_vel.z = 0.0;
            (*fp).self_vel.y = 0.0;
        }
    }
}

pub fn ftKb_PrSpecialAirNEndR_Coll(gobj: G) {
    const MF: u32 = (1 << 1) | (1 << 4) | (1 << 7) | (1 << 12) | (1 << 14) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27);
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80081D0C(gobj) != GA_Ground as i32 {
            ext::ftCommon_8007D7FC(fp);
            ftKb_SpecialNPr_801010D4(gobj, false, MF, (*fp).cur_anim_frame);
        }
    }
}

pub fn ftKb_PrSpecialNHit_Coll(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        if ext::ft_800824A0(gobj, &raw mut ftKb_Init_803CB758) {
            ftKb_PrRestoreScale(gobj);
            ext::ftCommon_8007D7FC(fp);
            if da.specialn_pr_freefall_toggle == 0.0 {
                st::ft_8008A2BC(gobj);
            } else {
                st::ftCo_LandingFallSpecial_Enter(gobj, 0, da.specialn_pr_freefall_toggle);
            }
        }
    }
}

/// `fn_80105978`: x21F8 callback; mirror the roll when the fighter is mirrored.
pub fn fn_80105978(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        rot_y(fp, 0, M_PI_2 * (*fp).facing_dir as f64);
        ftPr_MirrorSpecialN(fp);
    }
}
