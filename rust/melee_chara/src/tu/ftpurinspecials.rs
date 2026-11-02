//! melee/ft/kinds/ftPurin/ftpurinspecials.c: ports folded from the hand-written character modules (consolidation step 20).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftPr_SpecialS_Anim` (src/melee/ft/kinds/ftPurin/ftpurinspecials.c).
pub fn ftPr_SpecialS_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialS_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

/// `ftPr_SpecialAirS_Anim` (src/melee/ft/kinds/ftPurin/ftpurinspecials.c).
pub fn ftPr_SpecialAirS_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialAirS_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftPr_SpecialS_Phys` (src/melee/ft/kinds/ftPurin/ftpurinspecials.c).
pub fn ftPr_SpecialS_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialS_Phys");
    st::ft_80084FA8(gobj);
}

/// `ftPr_SpecialS_8013D590` (src/melee/ft/kinds/ftPurin/ftpurinspecials.c).
pub fn ftPr_SpecialS_8013D590(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialS_8013D590");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_GroundToAirStateChange(gobj, fp, 364 /* ftPr_MS_SpecialAirS */, 0xC4C508Au32 /* ftPr_MF_SpecialS_Coll */);
}

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftPr_SpecialS_Coll` (src/melee/ft/kinds/ftPurin/ftpurinspecials.c).
pub fn ftPr_SpecialS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialS_Coll");
    unsafe {
        if st::ft_800827A0(gobj) == 0 /* GA_Ground */ {
            crate::chars::r2::ftPurin::ftpurinspecials::ftPr_SpecialS_8013D590(gobj);
        }
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftPr_SpecialAirS_Coll` (src/melee/ft/kinds/ftPurin/ftpurinspecials.c).
pub fn ftPr_SpecialAirS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialAirS_Coll");
    unsafe {
        if (st::ft_80081D0C(gobj) != 0 /* GA_Ground */) {
            crate::chars::r6::ftPurin::ftpurinspecials::ftPr_SpecialS_8013D5F0(gobj);
        }
    }
}

/// `ftPr_SpecialS_8013D5F0` (src/melee/ft/kinds/ftPurin/ftpurinspecials.c).
pub fn ftPr_SpecialS_8013D5F0(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialS_8013D5F0");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 363 /* ftPr_MS_SpecialS */, 0xC4C508Au32 /* ftPr_MF_SpecialS_Coll */);
        st::ftCommon_ClampAirDrift(&mut *fp);
    }
}

/// `ftPr_SpecialS_Enter` (src/melee/ft/kinds/ftPurin/ftpurinspecials.c).
pub fn ftPr_SpecialS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialS_Enter");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, melee_types::ftPr_MS_SpecialS as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        st::Fighter_ClearCmdVars(&mut *fp);
    }
}

/// `ftPr_SpecialAirS_Enter` (src/melee/ft/kinds/ftPurin/ftpurinspecials.c).
pub fn ftPr_SpecialAirS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialAirS_Enter");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, melee_types::ftPr_MS_SpecialAirS as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        st::Fighter_ClearCmdVars(&mut *fp);
    }
}

pub fn ftPr_SpecialS_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftPr_SpecialS_IASA"); }
pub fn ftPr_SpecialAirS_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftPr_SpecialAirS_IASA"); }

/// `calcAngleRadians` (static): the launch angle from the stick's y (no fused site).
fn calc_angle_radians(fp: &Fighter, lstick_y: f32) -> f32 {
    let da = unsafe { &*(fp.dat_attrs as *const melee_types::ftPurinAttributes) };
    // stickGetDir(lstick_y, 0)
    let mut y = if lstick_y < 0.0 { -lstick_y } else { lstick_y };
    if y > da.xE0 {
        y = da.xE0;
    }
    y -= da.xDC;
    if y < 0.0 {
        y = 0.0;
    }
    if lstick_y < 0.0 {
        y = -y;
    }
    (y * da.xE4 / (da.xE0 - da.xDC)) * 0.01745329252f32
}

/// `ftPr_SpecialAirS_Phys` (src/melee/ft/kinds/ftPurin/ftpurinspecials.c): Pound in the air.
pub fn ftPr_SpecialAirS_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialAirS_Phys");
    unsafe {
        let fp = &mut *((*gobj).user_data as *mut Fighter);
        let da = &*(fp.dat_attrs as *const melee_types::ftPurinAttributes);
        if fp.cmd_vars[0] != 0 {
            fp.cmd_vars[0] = 0;
            fp.u.pr.x222C = fp.u.pr.x222C.wrapping_add(1);
            let angle = calc_angle_radians(fp, fp.input.lstick[0].y);
            fp.self_vel.y = da.xF0 * melee_runtime::gekko::sinf(angle);
            fp.self_vel.x = da.xF0 * (fp.facing_dir * melee_runtime::gekko::cosf(angle));
        }
        match fp.cmd_vars[1] {
            0 => st::ft_80084EEC(gobj),
            1 => {
                fp.self_vel.y = fp.self_vel.y * da.xF4;
                fp.self_vel.x = fp.self_vel.x * da.xF4;
            }
            2 => st::ft_80084DB0(gobj),
            _ => {}
        }
    }
}
