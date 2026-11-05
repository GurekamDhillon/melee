//! src/melee/ft/kinds/ftFox/ftfoxspecials.c: Fox Illusion / Falco Phantasm (side B).
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function). Smoke-tested by
//! tests/tu_ftfoxspecials.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::chars::da as da_g;
#[allow(unused_imports)]
use melee_fighter::ftcommon as co;
#[allow(unused_imports)]
use melee_types::{ftFox_DatAttrs, ftFx_MS_SpecialAirSEnd, ftFx_MS_SpecialSStart, Vec3};
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_fighter::ftcommon as fc;
use melee_types::*;

/// Already ported (rounds 2/3/4/6/7/11/12): re-exported under this TU.








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
    /// Like `hook!`, but an unset hook runs the native port `$port` (a ported TU elsewhere).
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
    crate::port_hook!(EFSYNC_SPAWN, efSync_Spawn, crate::adapt::efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut core::ffi::c_void]) -> *mut core::ffi::c_void);
    pub static mut IT_8029CEB4: Option<fn(*mut HSD_GObj, *const Vec3, ItemKind, f32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_8029CEB4(owner: *mut HSD_GObj, pos: *const Vec3, kind: ItemKind, facing_dir: f32) -> *mut HSD_GObj {
                match unsafe { IT_8029CEB4 } {
                    Some(f) => f(owner, pos, kind, facing_dir),
                    None => { return unsafe { melee_items::tu::itfoxillusion::it_8029CEB4(owner as _, pos as _, kind as _, facing_dir as _) as _ }; },
                }
            }
    hook_or!(FIGHTER_SETEFFECTHITLAGCALLBACKS, Fighter_SetEffectHitlagCallbacks, (fp: *mut Fighter),
        |fp: *mut Fighter| unsafe { melee_fighter::tu::ftCo_AttackS4::Fighter_SetEffectHitlagCallbacks(&mut *fp) });
    hook_or!(FTPARTGETROTX, ftPartGetRotX, (fp: *mut Fighter, part: i32) -> f32,
        |fp: *mut Fighter, part: i32| unsafe { melee_fighter::tu::ftparts::ftPartGetRotX(&*fp, part) });
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *const ftFox_DatAttrs {
    (*fp).dat_attrs as *const ftFox_DatAttrs
}

/// `ftGetTemplateKind` (ft/inlines.h, TARGET_PC).
fn ftGetTemplateKind(fp: &Fighter) -> i32 {
    let kind = fp.kind as i32;
    if kind >= Ft_Kind_Mex0 as i32 && melee_fighter::hooks::geno_define_template_skeleton(kind) != 0 {
        return melee_fighter::hooks::geno_define_base_kind(kind);
    }
    kind
}

pub unsafe extern "C" fn cb_ftFx_SpecialS_CreateGFX(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftFx_SpecialS_CreateGFX");
    ftFx_SpecialS_CreateGFX(g);
}

/// Create the Illusion / Phantasm effect.
pub fn ftFx_SpecialS_CreateGFX(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialS_CreateGFX");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            let joint = (*(*fp).parts.add(FtPart_TopN as usize)).joint;
            ext::efSync_Spawn(
                0x48D,
                gobj,
                &[joint as *mut core::ffi::c_void, &mut (*fp).facing_dir as *mut f32 as *mut core::ffi::c_void],
            );
            (*fp).set_x2219_b0(1);
        }
        ext::Fighter_SetEffectHitlagCallbacks(fp);
        (*fp).accessory4_cb = None;
    }
}

/// True when Fox / Falco is NOT in any Illusion / Phantasm motion state.
pub fn ftFx_SpecialS_CheckGhostRemove(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftFx_SpecialS_CheckGhostRemove");
    unsafe {
        let msid = (*fpp(gobj)).motion_id as i32;
        !(msid >= ftFx_MS_SpecialSStart as i32 && msid <= ftFx_MS_SpecialAirSEnd as i32)
    }
}

pub fn ftFx_SpecialS_GetCmdVar2(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftFx_SpecialS_GetCmdVar2");
    unsafe { (*fpp(gobj)).cmd_vars[2] as i32 }
}

/// `*ghostPos = fp->mv.fx.SpecialS.ghostEffectPos[index]` (index unchecked in C).
pub fn ftFx_SpecialS_CopyGhostPosIndexed(gobj: *mut HSD_GObj, index: i32, ghost_pos: *mut Vec3) {
    melee_core::ct!("ftFx_SpecialS_CopyGhostPosIndexed");
    unsafe { *ghost_pos = (*fpp(gobj)).mv.fx.SpecialS.ghostEffectPos[index as usize] }
}

pub fn ftFx_SpecialS_ReturnFloatVarIndexed(gobj: *mut HSD_GObj, index: i32) -> f32 {
    melee_core::ct!("ftFx_SpecialS_ReturnFloatVarIndexed");
    unsafe { (*fpp(gobj)).mv.fx.SpecialS.blendFrames[index as usize] }
}

pub fn ftFx_SpecialSStart_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialSStart_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        (*fp).cmd_vars[2] = 0;
        (*fp).mv.fx.SpecialS.gravityDelay = melee_core::ppc::fctiwz_f32((*da).x24_FOX_ILLUSION_GRAVITY_DELAY);
        (*fp).mv.fx.SpecialS.ghostGObj = core::ptr::null_mut();

        (*fp).gr_vel /= (*da).x28_FOX_ILLUSION_GROUND_VEL_X;

        st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialSStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftFx_SpecialAirSStart_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirSStart_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        (*fp).cmd_vars[2] = 0;
        (*fp).mv.fx.SpecialS.gravityDelay = melee_core::ppc::fctiwz_f32((*da).x24_FOX_ILLUSION_GRAVITY_DELAY);
        (*fp).mv.fx.SpecialS.ghostGObj = core::ptr::null_mut();

        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x /= (*da).x28_FOX_ILLUSION_GROUND_VEL_X;

        st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialAirSStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);

        (*fp).x1968_jumpsUsed = crate::chars::caps_max_jumps(&*fp) as u8;
    }
}

pub fn ftFx_SpecialSStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialSStart_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftFx_SpecialS_Enter(gobj);
    }
}

pub fn ftFx_SpecialAirSStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirSStart_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftFx_SpecialAirS_Enter(gobj);
    }
}

/// Gobj entry for the fighter-only port `chars::ftFox::ftfoxspecials::ftFx_SpecialAirSStart_Phys`.
pub fn ftFx_SpecialAirSStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirSStart_Phys");
    unsafe { crate::chars::ftFox::ftfoxspecials::ftFx_SpecialAirSStart_Phys(&mut *fpp(gobj)) }
}

/// `ftFox_SpecialS_CreateGhostItem` (static inline): spawn the ghost on the script's request.
pub fn ftFox_SpecialS_CreateGhostItem(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFox_SpecialS_CreateGhostItem");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[2] == 1 {
            (*fp).cmd_vars[2] = 0;
            let ghost_gobj = if ftGetTemplateKind(&*fp) == Ft_Kind_Fox as i32 {
                ext::it_8029CEB4(gobj, &(*fp).cur_pos, It_Kind_Fox_Illusion, (*fp).facing_dir)
            } else {
                ext::it_8029CEB4(gobj, &(*fp).cur_pos, It_Kind_Falco_Phantasm, (*fp).facing_dir)
            };
            if !ghost_gobj.is_null() {
                (*fp).mv.fx.SpecialS.ghostGObj = ghost_gobj;
                (*fp).set_x2222_b2(1);
            }
        }
    }
}

pub fn ftFx_SpecialS_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialS_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftFx_SpecialSEnd_Enter(gobj);
    }
    ftFox_SpecialS_CreateGhostItem(gobj);
}

pub fn ftFx_SpecialAirS_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirS_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftFx_SpecialAirSEnd_Enter(gobj);
    }
    ftFox_SpecialS_CreateGhostItem(gobj);
}

/// B during the dash ends it (grounded and aerial IASA are the same code in the C).
unsafe fn dash_iasa(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    if (*fp).input.pressed_buttons as i32 & HSD_PAD_B != 0 {
        if (*fp).ground_or_air == GA_Air {
            ftFx_SpecialAirSEnd_Enter(gobj);
            return;
        } else {
            ftFx_SpecialSEnd_Enter(gobj);
        }
    }
}

pub fn ftFx_SpecialS_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialS_IASA");
    unsafe { dash_iasa(gobj) }
}

pub fn ftFx_SpecialAirS_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirS_IASA");
    unsafe { dash_iasa(gobj) }
}

/// `ftFox_SpecialS_SetPhys` (static inline): shift the afterimage history, record this frame.
pub fn ftFox_SpecialS_SetPhys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFox_SpecialS_SetPhys");
    unsafe {
        let fp = fpp(gobj);
        let s = &mut (*fp).mv.fx.SpecialS;

        s.ghostEffectPos[3] = s.ghostEffectPos[2];
        s.ghostEffectPos[2] = s.ghostEffectPos[1];
        s.ghostEffectPos[1] = s.ghostEffectPos[0];
        s.ghostEffectPos[0] = (*fp).cur_pos;

        s.blendFrames[3] = s.blendFrames[2];
        s.blendFrames[2] = s.blendFrames[1];
        s.blendFrames[1] = s.blendFrames[0];
        s.blendFrames[0] = ext::ftPartGetRotX(fp, 0);
    }
}

pub fn ftFx_SpecialS_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialS_Phys");
    st::ft_80085088(gobj);
    ftFox_SpecialS_SetPhys(gobj);
}

pub fn ftFx_SpecialAirS_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirS_Phys");
    st::ft_80085134(gobj);
    ftFox_SpecialS_SetPhys(gobj);
}

/// `ftFox_SpecialS_SetVars` (static inline): seed the afterimage history.
pub fn ftFox_SpecialS_SetVars(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFox_SpecialS_SetVars");
    unsafe {
        let fp = fpp(gobj);
        let cur_pos = (*fp).cur_pos;
        for i in (0..4).rev() {
            (*fp).mv.fx.SpecialS.ghostEffectPos[i] = cur_pos;
        }

        let var = ext::ftPartGetRotX(fp, 0);
        for i in (0..4).rev() {
            (*fp).mv.fx.SpecialS.blendFrames[i] = var;
        }

        (*fp).accessory4_cb = Some(cb_ftFx_SpecialS_CreateGFX);
    }
}

pub fn ftFx_SpecialS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialS_Enter");
    st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialS as i32, Ft_MF_SkipRumble as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftFox_SpecialS_SetVars(gobj);
}

pub fn ftFx_SpecialAirS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirS_Enter");
    st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialAirS as i32, Ft_MF_SkipRumble as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftFox_SpecialS_SetVars(gobj);
}

pub fn ftFx_SpecialSEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialSEnd_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        if (*fp).mv.fx.SpecialS.gravityDelay != 0 {
            (*fp).mv.fx.SpecialS.gravityDelay -= 1;
        }
        fc::calc_ground_accel_deaccel(&mut *fp, (*da).x38_FOX_ILLUSION_GROUND_FRICTION);
        fc::set_self_movement_from_grounded_movement(&mut *fp);
        ftFox_SpecialS_SetPhys(gobj);
    }
}

pub fn ftFx_SpecialAirSEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirSEnd_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        if (*fp).mv.fx.SpecialS.gravityDelay != 0 {
            (*fp).mv.fx.SpecialS.gravityDelay -= 1;
        } else {
            fc::fall(&mut *fp, (*da).x48_FOX_ILLUSION_TERMINAL_VELOCITY, (*fp).co_attrs.terminal_velocity);
        }
        fc::calc_self_accel_deaccel(&mut *fp, (*da).x40_FOX_ILLUSION_AIR_MUL_X);
        ftFox_SpecialS_SetPhys(gobj);
    }
}

/// `ftFox_SpecialSEnd_SetVars` (static inline).
pub fn ftFox_SpecialSEnd_SetVars(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFox_SpecialSEnd_SetVars");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).mv.fx.SpecialS.gravityDelay = melee_core::ppc::fctiwz_f32((*da).x44_FOX_ILLUSION_FALL_ACCEL);
        (*fp).set_x2222_b2(1);
    }
}

pub fn ftFx_SpecialSEnd_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialSEnd_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        (*fp).gr_vel = (*da).x34_FOX_ILLUSION_GROUND_END_VEL_X * (*fp).facing_dir;

        st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialSEnd as i32, Ft_MF_SkipRumble as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ftFox_SpecialSEnd_SetVars(gobj);
    }
}

pub fn ftFx_SpecialAirSEnd_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirSEnd_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        (*fp).self_vel.x = (*da).x3C_FOX_ILLUSION_AIR_END_VEL_X * (*fp).facing_dir;
        (*fp).self_vel.y = 0.0;

        st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialAirSEnd as i32, Ft_MF_SkipRumble as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ftFox_SpecialSEnd_SetVars(gobj);
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftFx_SpecialAirSStart_Coll` (src/melee/ft/kinds/ftFox/ftfoxspecials.c).
pub fn ftFx_SpecialAirSStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirSStart_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_CheckGroundAndLedge(gobj, (if (*fp).facing_dir < 0.0f32 { -1i32 } else { 1i32 })) != 0) {
            crate::chars::r2::ftFox::ftfoxspecials::ftFx_SpecialAirSStart_AirToGround(gobj);
            return;
        }
        if (st::ftCliffCommon_80081298(gobj) != 0) {
            return;
        }
    }
}

/// `ftFx_SpecialAirS_Coll` (src/melee/ft/kinds/ftFox/ftfoxspecials.c).
pub fn ftFx_SpecialAirS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirS_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_CheckGroundAndLedge(gobj, (if (*fp).facing_dir < 0.0f32 { -1i32 } else { 1i32 })) != 0) {
            crate::chars::r4::ftFox::ftfoxspecials::ftFx_SpecialAirS_AirToGround(gobj);
            return;
        }
        if (st::ftCliffCommon_80081298(gobj) != 0) {
            return;
        }
    }
}

/// `ftFx_SpecialAirSEnd_Coll` (src/melee/ft/kinds/ftFox/ftfoxspecials.c).
pub fn ftFx_SpecialAirSEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirSEnd_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_CheckGroundAndLedge(gobj, (if (*fp).facing_dir < 0.0f32 { -1i32 } else { 1i32 })) != 0) {
            st::ftCo_LandingFallSpecial_Enter(gobj, 0 /* false */, ((*((*fp).dat_attrs as *const melee_types::ftFox_DatAttrs)).x50_FOX_ILLUSION_LANDING_LAG as f32));
            return;
        }
        if (st::ftCliffCommon_80081298(gobj) != 0) {
            return;
        }
    }
}

/// `ftFx_SpecialSStart_IASA` (src/melee/ft/kinds/ftFox/ftfoxspecials.c).
pub fn ftFx_SpecialSStart_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialSStart_IASA");
    unsafe {
        return;
    }
}

/// `ftFx_SpecialAirSStart_IASA` (src/melee/ft/kinds/ftFox/ftfoxspecials.c).
pub fn ftFx_SpecialAirSStart_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirSStart_IASA");
    unsafe {
        return;
    }
}

/// `ftFx_SpecialSEnd_IASA` (src/melee/ft/kinds/ftFox/ftfoxspecials.c).
pub fn ftFx_SpecialSEnd_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialSEnd_IASA");
    unsafe {
        return;
    }
}

/// `ftFx_SpecialAirSEnd_IASA` (src/melee/ft/kinds/ftFox/ftfoxspecials.c).
pub fn ftFx_SpecialAirSEnd_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirSEnd_IASA");
    unsafe {
        return;
    }
}

/// `ftFx_SpecialSStart_GroundToAir` (src/melee/ft/kinds/ftFox/ftfoxspecials.c).
pub fn ftFx_SpecialSStart_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialSStart_GroundToAir");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_8007D60C(fp);
    st::Fighter_ChangeMotionState(gobj, 350 /* ftFx_MS_SpecialAirSStart */, 0xC4C5880u32 /* ftFx_MF_SpecialS_Coll */, fp.cur_anim_frame, 1.0_f32, 0.0_f32, core::ptr::null_mut());
}

/// `ftFx_SpecialAirSStart_AirToGround` (src/melee/ft/kinds/ftFox/ftfoxspecials.c).
pub fn ftFx_SpecialAirSStart_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirSStart_AirToGround");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_AirToGroundStateChange(gobj, fp, 347 /* ftFx_MS_SpecialSStart */, 0xC4C5880u32 /* ftFx_MF_SpecialS_Coll */);
}

/// `ftFx_SpecialSEnd_Anim` (src/melee/ft/kinds/ftFox/ftfoxspecials.c).
pub fn ftFx_SpecialSEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialSEnd_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

/// `ftFx_SpecialSEnd_Coll` (src/melee/ft/kinds/ftFox/ftfoxspecials.c).
pub fn ftFx_SpecialSEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialSEnd_Coll");
    if st::ft_800827A0(gobj) == 0 /* false */ {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftFx_SpecialSStart_Coll` (src/melee/ft/kinds/ftFox/ftfoxspecials.c).
pub fn ftFx_SpecialSStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialSStart_Coll");
    unsafe {
        if st::ft_80082708(gobj) == 0 /* false */ {
            crate::chars::r2::ftFox::ftfoxspecials::ftFx_SpecialSStart_GroundToAir(gobj);
        }
    }
}

/// `ftFx_SpecialS_Coll` (src/melee/ft/kinds/ftFox/ftfoxspecials.c).
pub fn ftFx_SpecialS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialS_Coll");
    unsafe {
        if (st::ft_80082708(gobj) == 0 /* false */) {
            crate::chars::r4::ftFox::ftfoxspecials::ftFx_SpecialS_GroundToAir(gobj);
        }
    }
}

/// `ftFx_SpecialS_GroundToAir` (src/melee/ft/kinds/ftFox/ftfoxspecials.c).
pub fn ftFx_SpecialS_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialS_GroundToAir");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_8007D60C(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, 351 /* ftFx_MS_SpecialAirS */, 0xC4C5884u32 /* ftFx_MF_SpecialSDash_Coll */, (*fp).cur_anim_frame, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        (*fp).cmd_vars[2] = 0_u32;
    }
}

/// `ftFx_SpecialAirS_AirToGround` (src/melee/ft/kinds/ftFox/ftfoxspecials.c).
pub fn ftFx_SpecialAirS_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirS_AirToGround");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 348 /* ftFx_MS_SpecialS */, 0xC4C5884u32 /* ftFx_MF_SpecialSDash_Coll */);
        (*fp).cmd_vars[2] = 0_u32;
    }
}

/// `ftFx_SpecialSStart_Phys` (src/melee/ft/kinds/ftFox/ftfoxspecials.c).
pub fn ftFx_SpecialSStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialSStart_Phys");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (((*fp).mv.fx.SpecialS.gravityDelay as i32) != 0i32 /* 0 */) {
            (*fp).mv.fx.SpecialS.gravityDelay = (*fp).mv.fx.SpecialS.gravityDelay.wrapping_sub(1_i32);
        }
        st::ft_80084F3C(gobj);
    }
}

/// `ftFx_SpecialAirSEnd_Anim` (src/melee/ft/kinds/ftFox/ftfoxspecials.c).
pub fn ftFx_SpecialAirSEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirSEnd_Anim");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            st::ftCo_80096900(gobj, 1, 0, 1 /* true */, ((*((*fp).dat_attrs as *const melee_types::ftFox_DatAttrs)).x4C_FOX_ILLUSION_FREEFALL_MOBILITY as f32), ((*((*fp).dat_attrs as *const melee_types::ftFox_DatAttrs)).x50_FOX_ILLUSION_LANDING_LAG as f32));
        }
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftFx_SpecialAirSStart_Phys_fp(fp: &mut Fighter) {
    melee_core::ct!("ftFx_SpecialAirSStart_Phys_fp");
    let a: &ftFox_DatAttrs = da_g(fp);
    unsafe {
        if fp.mv.fx.SpecialS.gravityDelay != 0 {
            fp.mv.fx.SpecialS.gravityDelay -= 1;
        } else {
            co::fall(fp, a.x30_FOX_ILLUSION_UNK2, fp.co_attrs.terminal_velocity);
        }
    }
    co::calc_self_accel_deaccel(fp, a.x2C_FOX_ILLUSION_UNK1);
}

pub fn ftFx_SpecialS_CheckGhostRemove_fp(fp: &Fighter) -> bool {
    melee_core::ct!("ftFx_SpecialS_CheckGhostRemove_fp");
    let msid = fp.motion_id as i32;
    !(msid >= ftFx_MS_SpecialSStart as i32 && msid <= ftFx_MS_SpecialAirSEnd as i32)
}

/// `*ghostPos = fp->mv.fx.SpecialS.ghostEffectPos[index]` (index unchecked in C).
pub fn ftFx_SpecialS_CopyGhostPosIndexed_fp(fp: &Fighter, index: i32, ghost_pos: &mut Vec3) {
    melee_core::ct!("ftFx_SpecialS_CopyGhostPosIndexed_fp");
    *ghost_pos = unsafe { fp.mv.fx.SpecialS.ghostEffectPos[index as usize] };
}

pub fn ftFx_SpecialS_GetCmdVar2_fp(fp: &Fighter) -> i32 {
    melee_core::ct!("ftFx_SpecialS_GetCmdVar2_fp");
    fp.cmd_vars[2] as i32
}
