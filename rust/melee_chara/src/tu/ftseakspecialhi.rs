//! src/melee/ft/kinds/ftSeak/ftseakspecialhi.c: Sheik's Up B (Vanish): the start states, the
//! travel states (grounded and aerial), the fall and landing states, and the accessory
//! callbacks. Fused sites (tools/rust-port/fma_sites.py, C IR `llvm.fmuladd`): the `x44 * m + x48`
//! of `ftSk_SpecialHi_80113838` (1) and `ftSk_SpecialHi_80113A30` (2), each as `fmadds` before the
//! `cosf` / `sinf` multiply. `ftSk_SpecialHi_80113E40`, `ftSk_SpecialHi_Coll`,
//! `ftSk_SpecialAirHi_Coll`, `ftSk_SpecialHi_Enter`, `ftSk_SpecialAirHi_Enter`,
//! `ftSk_SpecialHi_Anim`, `ftSk_SpecialHi_Phys` and `ftSk_SpecialHiStart_0_Phys` are the existing
//! ports in `chars::r*`, re-exported here. Callees in TUs not yet ported are `ext` hooks named
//! as the C function; unset, they panic. `ScriptGame_CapsMaxJumps` is the PC build's
//! `FT_CAPS_MAX_JUMPS`; its default is `co_attrs.max_jumps` (no script loaded).
//! State oracle (tests/tu_ftseakspecialhi.rs, C data from gen/turef/tu_ftseakspecialhi_ref.c):
//! ftSk_SpecialHi_80113EAC, ftSk_SpecialHi_80113F68 (with their inlines), ftSk_SpecialHi_801137C8,
//! ftSk_SpecialHiStart_1_Anim, ftSk_SpecialAirHiStart_1_Anim. The other functions are unverified.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::all)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_runtime::gekko::{cosf, sinf};
use melee_runtime::lbtrigf::atan2f;
use melee_types::*;

/// Already ported (rounds 2-11): re-exported under this TU.
pub use crate::chars::r11::ftSeak::ftseakspecialhi::ftSk_SpecialAirHi_Coll;
pub use crate::chars::r2::ftSeak::ftseakspecialhi::{ftSk_SpecialHiStart_0_Phys, ftSk_SpecialHi_Phys};
pub use crate::chars::r3::ftSeak::ftseakspecialhi::{ftSk_SpecialHi_80113E40, ftSk_SpecialHi_Coll};
pub use crate::chars::r4::ftSeak::ftseakspecialhi::{ftSk_SpecialAirHi_Enter, ftSk_SpecialHi_Enter};
pub use crate::chars::r8::ftSeak::ftseakspecialhi::ftSk_SpecialHi_Anim;

/// Calls into TUs that are not ported yet (or that have no native port reachable from here).
/// Each is a settable hook; an unset hook panics, except where `hook_or!` names a native port.
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
    /// Like `hook!`, but an unset hook runs the native port `$port`.
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
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(j: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(j, off, out),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(j as _, off as _, out as _); } return; },
                }
            }
    pub static mut IT_802B1C60: Option<fn(*mut HSD_GObj, *mut Vec3, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802B1C60(gobj: *mut HSD_GObj, pos: *mut Vec3, facing_dir: f32)  {
                match unsafe { IT_802B1C60 } {
                    Some(f) => f(gobj, pos, facing_dir),
                    None => { unsafe { let _ = melee_items::tu::itseakvanish::it_802B1C60(gobj as _, pos as _, facing_dir as _); } return; },
                }
            }
    crate::port_hook!(EFSYNC_SPAWN, efSync_Spawn, crate::adapt::efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut core::ffi::c_void]) -> *mut core::ffi::c_void);
    crate::port_hook!(FTCOMMON_8007DA24, ftCommon_8007DA24, crate::adapt::ftCommon_8007DA24, (fp: &mut Fighter));
    crate::port_hook!(FTCOLL_8007B62C, ftColl_8007B62C, crate::adapt::ftColl_8007B62C, (g: *mut HSD_GObj, s: i32));
    crate::port_hook!(SCRIPTGAME_CAPSMAXJUMPS, ScriptGame_CapsMaxJumps, crate::adapt::ScriptGame_CapsMaxJumps, (fp: &Fighter) -> i32);
    hook_or!(FT_PLAYSFX, ft_PlaySFX, (fp: &mut Fighter, sfx: i32, vol: i32, pan: i32),
        |fp: &mut Fighter, sfx: i32, vol: i32, pan: i32| melee_fighter::hooks::play_sfx(fp, sfx, vol, pan));
    hook_or!(FTCO_8009A134, ftCo_8009A134, (g: *mut HSD_GObj) -> bool,
        melee_fighter::tu::ftCo_Pass::ftCo_8009A134);
    hook_or!(FIGHTER_SETEFFECTHITLAGCALLBACKS, Fighter_SetEffectHitlagCallbacks, (fp: &mut Fighter),
        melee_fighter::tu::ftCo_AttackS4::Fighter_SetEffectHitlagCallbacks);
}

/// `ftSk_MF_SpecialHi_Coll`: `ftCommon_GroundAirColl_MF | Ft_MF_KeepGfx |
/// Ft_MF_KeepColAnimHitStatus | Ft_MF_SkipHit`.
pub const ftSk_MF_SpecialHi_Coll: u32 = Ft_MF_SkipMatAnim | Ft_MF_SkipColAnim | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis | Ft_MF_Unk19 | Ft_MF_SkipModelPartVis | Ft_MF_SkipModelFlags | Ft_MF_Unk27
    | Ft_MF_KeepGfx | Ft_MF_KeepColAnimHitStatus | Ft_MF_SkipHit;

/// Flags of `ftSk_SpecialHi_8011374C` and `ftSk_SpecialHi_801137C8`: `0xC4C` plus `0x508E`.
/// `Collide_CeilingMask` (mp/forward.h): `Collide_CeilingPush | Collide_CeilingHug`.
const COLLIDE_CEILING_MASK: i32 = Collide_CeilingPush | Collide_CeilingHug;

const MF_GROUND_TO_AIR_FALL: u32 = Ft_MF_SkipModelPartVis | Ft_MF_SkipItemVis | Ft_MF_Unk19
    | Ft_MF_SkipModelFlags | Ft_MF_Unk27
    | Ft_MF_KeepGfx | Ft_MF_KeepColAnimHitStatus | Ft_MF_SkipHit | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim | Ft_MF_UpdateCmd;

/// `MTXDegToRad(x)` (`x * 0.01745329252f`).
#[inline(always)]
fn mtx_deg_to_rad(x: f32) -> f32 {
    x * 0.01745329252_f32
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *const ftSeakAttributes {
    (*fp).dat_attrs as *const ftSeakAttributes
}

/// `lbVector_AngleXY(a, b)` (lb/lbvector.c, verified in melee_runtime) over the generated `Vec3`.
#[inline(always)]
fn angle_xy(a: &Vec3, b: &Vec3) -> f32 {
    // SAFETY: both are `{ f32 x, y, z }` repr(C) records.
    unsafe {
        melee_runtime::lbvector::angle_xy(
            &*(a as *const Vec3 as *const melee_core::vec::Vec3),
            &*(b as *const Vec3 as *const melee_core::vec::Vec3),
        )
    }
}

/// `ftGetFacingDirInt` (ft/inlines.h).
#[inline(always)]
fn ft_get_facing_dir_int(fp: &Fighter) -> i32 {
    if fp.facing_dir < 0.0 { -1 } else { 1 }
}

/// `my_sqrtf` (the file's static inline; `crate::pure::my_sqrtf`).
#[inline(always)]
fn my_sqrtf(x: f32) -> f32 {
    crate::pure::my_sqrtf(x)
}

/// The hip joint's position, as `lb_8000B1CC(fp->parts[FtPart_HipN].joint, NULL, &pos)` gives it.
#[inline(always)]
unsafe fn hip_pos(fp: *mut Fighter) -> Vec3 {
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let joint = (*(*fp).parts.add(FtPart_HipN as usize)).joint;
    ext::lb_8000B1CC(joint, core::ptr::null(), &mut pos);
    pos
}

/// `ftCommon_HandleTeleportCollisions` (ft/inlines.h, static inline): the three env checks.
unsafe fn handle_teleport_collisions(
    gobj: *mut HSD_GObj,
    fp: *mut Fighter,
    coll: *mut CollData,
    angle_clamp: *const i32,
    on_collide: unsafe extern "C" fn(*mut HSD_GObj),
) {
    let limit = |a: i32| mtx_deg_to_rad(90.0_f32 + a as f32);
    let env = (*coll).env_flags;
    if (env & COLLIDE_CEILING_MASK) != 0 && angle_xy(&(*coll).ceiling.normal, &(*fp).self_vel) > limit(*angle_clamp) {
        on_collide(gobj);
    }
    let env = (*coll).env_flags;
    if (env & Collide_LeftWallMask) != 0
        && angle_xy(&(*coll).left_facing_wall.normal, &(*fp).self_vel) > limit(*angle_clamp)
    {
        on_collide(gobj);
    }
    let env = (*coll).env_flags;
    if (env & Collide_RightWallMask) != 0
        && angle_xy(&(*coll).right_facing_wall.normal, &(*fp).self_vel) > limit(*angle_clamp)
    {
        on_collide(gobj);
    }
}

/// `fn_80112ED8` (static): the accessory callback after the Vanish starts.
pub fn fn_80112ED8(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            ftSk_SpecialHi_80112F48(gobj);
            ftSk_SpecialHi_80112FA8(gobj);
            ext::ft_PlaySFX(&mut *fp, 115, 0x7F, 0x40);
        }
        (*fp).accessory4_cb = None;
    }
}

/// `ftSk_SpecialHi_80112F48` (static): spawn the vanish item at the hip.
pub fn ftSk_SpecialHi_80112F48(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let mut pos = hip_pos(fp);
        pos.z = 0.0;
        ext::it_802B1C60(gobj, &mut pos, (*fp).facing_dir);
    }
}

/// `ftSk_SpecialHi_80112FA8` (static): the hip effect, once per vanish.
pub fn ftSk_SpecialHi_80112FA8(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let mut pos = hip_pos(fp);
        if (*fp).x2219_b0() == 0 {
            ext::efSync_Spawn(1284, gobj, &[&mut pos as *mut Vec3 as *mut core::ffi::c_void]);
            (*fp).set_x2219_b0(1);
        }
        ext::Fighter_SetEffectHitlagCallbacks(&mut *fp);
    }
}

/// `fn_80113038` (static): the accessory callback while travelling.
pub fn fn_80113038(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            let mut pos = hip_pos(fp);
            ext::efSync_Spawn(1285, gobj, &[&mut pos as *mut Vec3 as *mut core::ffi::c_void]);
            (*fp).set_x2219_b0(1);
        }
        ext::Fighter_SetEffectHitlagCallbacks(&mut *fp);
        (*fp).accessory4_cb = None;
    }
}

/// The C `fn_80112ED8` / `fn_80113038` / `ftSk_SpecialHi_80113F68` as `HSD_GObjEvent` callbacks.
pub unsafe extern "C" fn cb_fn_80112ED8(g: *mut HSD_GObj) {
    fn_80112ED8(g);
}
pub unsafe extern "C" fn cb_fn_80113038(g: *mut HSD_GObj) {
    fn_80113038(g);
}
pub unsafe extern "C" fn cb_ftSk_SpecialHi_80113F68(g: *mut HSD_GObj) {
    ftSk_SpecialHi_80113F68(g);
}

/// `ftSk_SpecialHiStart_0_Anim`.
pub fn ftSk_SpecialHiStart_0_Anim(gobj: *mut HSD_GObj) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftSk_SpecialHi_80113838(gobj);
    }
}

/// `ftSk_SpecialAirHiStart_0_Anim`.
pub fn ftSk_SpecialAirHiStart_0_Anim(gobj: *mut HSD_GObj) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftSk_SpecialHi_80113A30(gobj);
    }
}

pub fn ftSk_SpecialHiStart_0_IASA(_gobj: *mut HSD_GObj) {}

/// Interrupt_SheikUpBStartAir
pub fn ftSk_SpecialAirHiStart_0_IASA(_gobj: *mut HSD_GObj) {}

/// `ftSk_SpecialAirHiStart_0_Phys`: fall with the file's fall limits, then the drift.
pub fn ftSk_SpecialAirHiStart_0_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = da(fp);
        melee_fighter::ftcommon::fall(&mut *fp, (*attributes).x30, (*attributes).x34);
        melee_fighter::ftcommon::calc_self_accel_drift(&mut *fp);
    }
}

/// `ftSk_SpecialHiStart_0_Coll`.
pub fn ftSk_SpecialHiStart_0_Coll(gobj: *mut HSD_GObj) {
    if st::ft_80082708(gobj) == GA_Ground as i32 {
        ftSk_SpecialHi_80113324(gobj);
    }
}

/// `ftSk_SpecialAirHiStart_0_Coll`.
pub fn ftSk_SpecialAirHiStart_0_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let ground_or_ledge = st::ft_CheckGroundAndLedge(gobj, ft_get_facing_dir_int(&*fp)) != 0;
        if ground_or_ledge {
            ftSk_SpecialHi_80113390(gobj);
        } else if st::ftCliffCommon_80081298(gobj) != 0 {
            return;
        }
    }
}

/// `ftSk_SpecialHi_80113324` (static): the grounded Vanish landing.
pub fn ftSk_SpecialHi_80113324(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D60C(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, 0x166, 0x0C4C508E, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        (*fp).accessory4_cb = Some(cb_fn_80112ED8);
    }
}

/// `ftSk_SpecialHi_80113390` (static): the aerial Vanish landing.
pub fn ftSk_SpecialHi_80113390(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 0x163, ftSk_MF_SpecialHi_Coll);
        (*fp).accessory4_cb = Some(cb_fn_80112ED8);
    }
}

/// `ftSk_SpecialHiStart_1_Anim`: the grounded travel timer (reads `mv.sk.specialn.x0`).
pub fn ftSk_SpecialHiStart_1_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.sk.specialn.x0 -= 1;
        if 0 >= (*fp).mv.sk.specialn.x0 {
            ftSk_SpecialHi_80113EAC(gobj);
        }
    }
}

/// `ftSk_SpecialAirHiStart_1_Anim`: the aerial travel timer.
pub fn ftSk_SpecialAirHiStart_1_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.sk.specialhi.x0 -= 1;
        if (*fp).mv.sk.specialhi.x0 <= 0 {
            ftSk_SpecialHi_80113F68(gobj);
        }
    }
}

pub fn ftSk_SpecialHiStart_1_IASA(_gobj: *mut HSD_GObj) {}

/// Interrupt_SheikUpBTravelAir
pub fn ftSk_SpecialAirHiStart_1_IASA(_gobj: *mut HSD_GObj) {}

/// `ftSk_SpecialHiStart_1_Phys`.
pub fn ftSk_SpecialHiStart_1_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        melee_fighter::ftcommon::set_self_movement_from_grounded_movement(&mut *fpp(gobj));
    }
}

/// `ftSk_SpecialAirHiStart_1_Phys` (empty in the C).
pub fn ftSk_SpecialAirHiStart_1_Phys(_gobj: *mut HSD_GObj) {}

/// `ftSk_SpecialHiStart_1_Coll`.
pub fn ftSk_SpecialHiStart_1_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            let env = (*fp).coll_data.env_flags;
            if (env & Collide_LeftWallMask) != 0 || (env & Collide_RightWallMask) != 0 {
                st::ftCommon_8007D60C(&mut *fp);
                ftSk_SpecialHi_80113F68(gobj);
                return;
            }
            ftSk_SpecialHi_8011374C(gobj);
            return;
        }
        let env = (*fp).coll_data.env_flags;
        if (env & Collide_LeftWallMask) != 0 || (env & Collide_RightWallMask) != 0 {
            ftSk_SpecialHi_80113EAC(gobj);
        }
    }
}

/// `ftSk_SpecialAirHiStart_1_Coll`.
pub fn ftSk_SpecialAirHiStart_1_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attr = da(fp);
        (*fp).mv.sk.specialhi.xC += 1;

        if st::ft_CheckGroundAndLedge(gobj, ft_get_facing_dir_int(&*fp)) != 0 {
            let fp2 = fpp(gobj);
            let attr2 = da(fp2);
            let var_r0 = if ((*fp2).mv.sk.specialhi.xC as f32) >= (*attr2).x3C {
                1
            } else if ext::ftCo_8009A134(gobj) {
                0
            } else {
                1
            };
            if var_r0 != 0 {
                ftSk_SpecialHi_801137C8(gobj);
                return;
            }
        }

        if st::ftCliffCommon_80081298(gobj) == 0 {
            let coll: *mut CollData = &mut (*fp).coll_data;
            handle_teleport_collisions(gobj, fp, coll, &(*attr).x50, cb_ftSk_SpecialHi_80113F68);
        }
    }
}

/// `ftSk_SpecialHi_8011374C` (static): AS_SheikUpBGroundtoAirFall.
pub fn ftSk_SpecialHi_8011374C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D60C(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, 0x167, MF_GROUND_TO_AIR_FALL, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
        (*fp).set_x2223_b4(1);
        (*fp).set_invisible(1);
    }
}

/// `ftSk_SpecialHi_801137C8` (static): AS_SheikUpBLand.
pub fn ftSk_SpecialHi_801137C8(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D7FC(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, 0x164, MF_GROUND_TO_AIR_FALL, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
        (*fp).set_invisible(1);
    }
}

/// `inlineA0` (static): the shared setup of the two travel states.
unsafe fn inline_a0(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let attributes = da(fp);
    (*fp).mv.sk.specialhi.x0 = (*attributes).x38;
    (*fp).x1968_jumpsUsed = ext::ScriptGame_CapsMaxJumps(&*fp) as u8;
    (*fp).set_x2223_b4(1);
    ext::ftColl_8007B62C(gobj, 2);
    (*fp).set_invisible(1);
    (*fp).accessory4_cb = Some(cb_fn_80112ED8);
}

/// `ftSk_SpecialHi_80113838` (static): AS_SheikUpBTravelGround.
pub fn ftSk_SpecialHi_80113838(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = da(fp);
        let coll: *mut CollData = &mut (*fp).coll_data;
        let mut stick_x = (*fp).input.lstick[0].x;
        let mut stick_y = (*fp).input.lstick[0].y;
        stick_x = stick_x * stick_x;
        stick_y = stick_y * stick_y;

        let mut stick_mag = my_sqrtf(stick_x + stick_y);
        if stick_mag > 1.0 {
            stick_mag = 1.0;
        }

        if !(stick_mag < (*attributes).x40) {
            let lstick = Vec3 { x: (*fp).input.lstick[0].x, y: (*fp).input.lstick[0].y, z: 0.0 };
            if !(angle_xy(&(*coll).floor.normal, &lstick) < core::f64::consts::FRAC_PI_2 as f32) {
                if !ext::ftCo_8009A134(gobj) {
                    st::ftCommon_UpdateFacing(&mut *fp);
                    let temp_f1_5 = atan2f((*fp).input.lstick[0].y, (*fp).input.lstick[0].x * (*fp).facing_dir);
                    (*fp).mv.sk.specialhi.vel.x = lstick.x;
                    (*fp).mv.sk.specialhi.vel.y = lstick.y;
                    let temp_f6 = fmadds((*attributes).x44, stick_mag, (*attributes).x48) * cosf(temp_f1_5);
                    (*fp).gr_vel = (*fp).facing_dir * temp_f6;
                    st::Fighter_ChangeMotionState(gobj, 0x164, 0, 35.0, 1.0, 0.0, core::ptr::null_mut());
                    st::ftAnim_8006EBA4(gobj);
                    st::ftAnim_SetAnimRate(gobj, 0.0);
                    inline_a0(gobj);
                    return;
                }
            }
        }
        st::ftCommon_8007D60C(&mut *fp);
        ftSk_SpecialHi_80113A30(gobj);
    }
}

/// `ftSk_SpecialHi_80113A30` (static): AS_SheikUpBTravelAir.
pub fn ftSk_SpecialHi_80113A30(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = da(fp);

        let stick_x0 = (*fp).input.lstick[0].x;
        let stick_y0 = (*fp).input.lstick[0].y;
        let stick_x = stick_x0 * stick_x0;
        let stick_y_sq = stick_y0 * stick_y0;
        let stick_mag = my_sqrtf(stick_x + stick_y_sq);

        let mut var_f31 = stick_mag;
        if stick_mag > 1.0 {
            var_f31 = 1.0;
        }
        let var_f30: f32;
        if var_f31 > (*attributes).x40 {
            let mut var_f1 = (*fp).input.lstick[0].x;
            if var_f1 < 0.0 {
                var_f1 = -var_f1;
            }
            if var_f1 > 0.001 {
                st::ftCommon_UpdateFacing(&mut *fp);
            }
            var_f30 = atan2f((*fp).input.lstick[0].y, (*fp).input.lstick[0].x * (*fp).facing_dir);
            (*fp).mv.sk.specialhi.vel.x = (*fp).input.lstick[0].x;
            (*fp).mv.sk.specialhi.vel.y = (*fp).input.lstick[0].y;
        } else {
            ext::ftCommon_8007DA24(&mut *fp);
            var_f30 = (core::f64::consts::PI / 2.0) as f32;
            (*fp).mv.sk.specialhi.vel.x = 0.0;
            (*fp).mv.sk.specialhi.vel.y = 1.0;
            var_f31 = 1.0;
        }
        (*fp).self_vel.x = (*fp).facing_dir * (fmadds((*attributes).x44, var_f31, (*attributes).x48) * cosf(var_f30));
        (*fp).self_vel.y = fmadds((*attributes).x44, var_f31, (*attributes).x48) * sinf(var_f30);
        st::Fighter_ChangeMotionState(gobj, 0x167, 0, 35.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        st::ftAnim_SetAnimRate(gobj, 0.0);
        inline_a0(gobj);
    }
}

/// `ftSk_SpecialAirHi_Anim`: the aerial landing check at the end of the travel.
pub fn ftSk_SpecialAirHi_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = da(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let x = (*attributes).x58;
            let y = (*attributes).x5C;
            st::ftCo_80096900(gobj, 1, 0, 1, x, y);
        }
    }
}

pub fn ftSk_SpecialHi_IASA(_gobj: *mut HSD_GObj) {}

/// Interrupt_SheikUpBEndAir
pub fn ftSk_SpecialAirHi_IASA(_gobj: *mut HSD_GObj) {}

/// `ftSk_SpecialAirHi_Phys`: the aerial Vanish physics (the fall gravity is halved in y).
pub fn ftSk_SpecialAirHi_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = da(fp);
        if (*fp).cmd_vars[0] != 0 {
            melee_fighter::ftcommon::fall_basic(&mut *fp);
            melee_fighter::ftcommon::clamp_self_vel_x(&mut *fp, (*attributes).x4C * (*fp).co_attrs.air_drift_max);
            return;
        } else {
            let vel_y = (*fp).self_vel.y;
            (*fp).self_vel.y = vel_y - (vel_y / 10.0);
        }
        melee_fighter::ftcommon::calc_self_accel_deaccel_air(&mut *fp);
    }
}

/// `ftSk_SpecialHi_80113EAC_inline` (static).
unsafe fn ftSk_SpecialHi_80113EAC_inline(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    (*fp).mv.co.walk.fast_anim_frame = (*fp).self_vel.x;
    (*fp).mv.co.common.x14 = (*fp).self_vel.y;
    (*fp).mv.co.common.x18 = (*fp).gr_vel;
    (*fp).gr_vel = 0.0;
    (*fp).self_vel.x = 0.0;
    (*fp).self_vel.y = 0.0;
    (*fp).set_invisible(0);
    (*fp).accessory4_cb = Some(cb_fn_80113038);
}

/// `ftSk_SpecialHi_80113EAC` (static): the grounded Vanish travel after the wall.
pub fn ftSk_SpecialHi_80113EAC(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = da(fp);
        st::Fighter_ChangeMotionState(gobj, 0x165, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        ftSk_SpecialHi_80113EAC_inline(gobj);
        (*fp).gr_vel = (*fp).mv.co.common.x18 * (*attributes).x54;
    }
}

/// `ftSk_SpecialHi_80113F68_inline` (static).
unsafe fn ftSk_SpecialHi_80113F68_inline(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    (*fp).mv.co.walk.fast_anim_frame = (*fp).self_vel.x;
    (*fp).mv.co.common.x14 = (*fp).self_vel.y;
    (*fp).mv.co.common.x18 = (*fp).gr_vel;
    (*fp).gr_vel = 0.0;
    (*fp).self_vel.x = 0.0;
    (*fp).self_vel.y = 0.0;
    (*fp).set_invisible(0);
    (*fp).accessory4_cb = Some(cb_fn_80113038);
}

/// `ftSk_SpecialHi_80113F68` (static): AS_SheikUpBFall.
pub fn ftSk_SpecialHi_80113F68(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = da(fp);
        st::Fighter_ChangeMotionState(gobj, ftSk_MS_SpecialAirHi as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        ftSk_SpecialHi_80113F68_inline(gobj);
        (*fp).self_vel.x = (*fp).mv.co.walk.fast_anim_frame * (*attributes).x54;
        (*fp).self_vel.y = (*fp).mv.co.common.x14 * (*attributes).x54;
    }
}
