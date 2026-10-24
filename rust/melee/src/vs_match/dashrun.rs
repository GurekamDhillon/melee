//! Dash and Run (ftCo_Dash.c, ftCo_Run.c): the ground dash out of a stick tap, and the run it
//! turns into once the dash window closes. Turn-around into a smash turn / TurnRun, RunBrake and
//! every button interrupt are not ported (they stop the run).

use super::*;

pub const FTCO_MS_DASH: i32 = 20;
pub const FTCO_MS_RUN: i32 = 21;

#[derive(Clone, Copy, Debug, Default)]
pub struct DashVars {
    /// `mv.co.dash.x0`: the initial-velocity kick, spent by the first Phys.
    pub x0: f32,
    /// `mv.co.dash.x4`: entered from a stick tap (the first IASA branch).
    pub x4: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RunVars {
    /// `mv.co.run.x0`: frames before the run can brake.
    pub x0: f32,
    pub x4: f32,
}

pub fn is_dash_or_run(m: i32) -> bool {
    m == FTCO_MS_DASH || m == FTCO_MS_RUN
}

/// `ftCo_Dash_CheckInput`.
pub fn dash_check_input(p: &mut Player, rand: &mut HsdRand) -> Result<bool, String> {
    let cd = melee_fighter::hooks::common_data();
    let fp = &*p.fp;
    let x = fp.input.lstick[0].x;
    if fc::abs_c(x) >= cd.dash_smash_stick_threshold && (fp.active_timer.lstick.x as i32) < cd.dash_smash_window as i32 {
        if x * fp.facing_dir < 0.0 {
            return Err(format!("player {}: ftCo_Turn_Enter_Smash not ported", p.slot + 1));
        }
        dash_enter(p, 1, rand)?;
        return Ok(true);
    }
    Ok(false)
}

/// `ftCo_Dash_Enter`.
pub fn dash_enter(p: &mut Player, arg1: i32, rand: &mut HsdRand) -> Result<(), String> {
    p.fp.cmd_vars[0] = 0;
    let row = load_row(p, melee_types::ftCo_SM_Dash as u32)?;
    p.keep_gfx = true; // ftCo_MF_Dash has Ft_MF_KeepGfx
    change_motion_state(p, FTCO_MS_DASH, row.as_ref(), rand)?;
    anim_step(p, rand)?;
    let fp = &mut *p.fp;
    fp.active_timer.lstick.x = 0xFE;
    let init_vel = fp.facing_dir * fp.co_attrs.dash_initial_velocity;
    p.dash.x0 = if fp.gr_vel * fp.facing_dir < 0.0 { init_vel } else { init_vel - fp.gr_vel };
    melee_fighter::ftcommon2::ftcommon_800804a0(fp, p.dash.x0);
    p.dash.x4 = arg1;
    if !fp.x197C.is_null() {
        return Err("ftCo_Dash_Enter: ft_PlaySFX (x197C) not ported".into());
    }
    Ok(())
}

/// `ftCo_Dash_Anim`.
pub fn dash_anim(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    if !p.skel.frames_remaining() {
        ft_8008a2bc(p, rand)?;
    }
    Ok(())
}

/// The checks both IASAs share that need a button, the C-stick or a trigger: any of them stops
/// the run (SpecialS / item / catch / dash attack / smash / escape / appeal).
fn press_gate(p: &Player, what: &str) -> Result<(), String> {
    let fp = &*p.fp;
    let i = &fp.input;
    if !fp.item_gobj.is_null() {
        return Err(format!("{what} with an item not ported"));
    }
    if i.pressed_buttons & !(PAD_STICK_DIRS | input::HSD_PAD_XY) != 0
        || i.cstick[0].x != 0.0
        || i.cstick[0].y != 0.0
        || i.triggers[0] != 0.0
        || i.held_buttons[0] & (input::HSD_PAD_A | input::HSD_PAD_Z | input::HSD_PAD_LR) != 0
    {
        return Err(format!(
            "player {}: {what} with a button / C-stick / trigger (pressed {:#x} held {:#x} c ({}, {}) trig {}): not ported",
            p.slot + 1,
            i.pressed_buttons,
            i.held_buttons[0],
            i.cstick[0].x,
            i.cstick[0].y,
            i.triggers[0]
        ));
    }
    Ok(())
}

/// `fn_800CAF78`: tap jump (relaxed threshold) or X / Y.
fn relaxed_jump(p: &mut Player, rand: &mut HsdRand) -> Result<bool, String> {
    let cd = melee_fighter::hooks::common_data();
    let fp = &*p.fp;
    if fp.input.lstick[0].y >= cd.relaxed_tap_jump_threshold && (fp.active_timer.lstick.y as i32) < cd.tap_jump_window as i32 {
        kneebend_enter(p, JUMP_INPUT_LSTICK, rand)?;
        return Ok(true);
    }
    if fp.input.pressed_buttons & input::HSD_PAD_XY != 0 {
        kneebend_enter(p, JUMP_INPUT_XY, rand)?;
        return Ok(true);
    }
    Ok(false)
}

/// `fn_800CA5F0`: stick forward past the run threshold -> `ftCo_Run_Enter(0)`.
fn run_check_input(p: &mut Player, rand: &mut HsdRand) -> Result<bool, String> {
    let cd = melee_fighter::hooks::common_data();
    let fp = &*p.fp;
    if fp.input.lstick[0].x * fp.facing_dir >= cd.x58_someLStickXThreshold {
        run_enter(p, 0.0, rand)?;
        return Ok(true);
    }
    Ok(false)
}

/// `ftCo_Run_Enter(gobj, arg0)`.
fn run_enter(p: &mut Player, arg0: f32, rand: &mut HsdRand) -> Result<(), String> {
    let row = load_row(p, melee_types::ftCo_SM_Run as u32)?;
    change_motion_state(p, FTCO_MS_RUN, row.as_ref(), rand)?;
    p.run.x0 = arg0;
    p.run.x4 = p.fp.gr_vel;
    Ok(())
}

/// `ftCo_Dash_IASA`.
pub fn dash_iasa(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let cd = melee_fighter::hooks::common_data();
    let frame = p.fp.cur_anim_frame;
    if p.dash.x4 != 0 && frame <= cd.x44 {
        if p.fp.x688 == 0 {
            return Err(format!("player {}: ftCo_SpecialS_CheckInput (Dash) not ported", p.slot + 1));
        }
        press_gate(p, "ftCo_Dash_IASA (dash window)")?;
        return dash_block_42(p, rand);
    } else if frame <= cd.x4C {
        if p.fp.x688 == 0 {
            return Err(format!("player {}: ftCo_SpecialS_CheckInput (Dash) not ported", p.slot + 1));
        }
        press_gate(p, "ftCo_Dash_IASA (turn window)")?;
        // ftCo_Dash_CheckInput on a backwards tap, else ftCo_80091AD8 (the shield) -> block_42.
        let fp = &*p.fp;
        if fp.input.lstick[0].x * fp.facing_dir < 0.0 && dash_check_input(p, rand)? {
            return dash_friction(p);
        }
        return dash_block_42(p, rand);
    }
    // After the window: ftCo_800D8A38, ftCo_Dash_CheckInput, the shield, then block_42.
    press_gate(p, "ftCo_Dash_IASA")?;
    if dash_check_input(p, rand)? {
        return dash_friction(p);
    }
    dash_block_42(p, rand)
}

/// `block_42` of `ftCo_Dash_IASA`: appeal, jump, then into the run once `cmd_vars[0]` says so.
fn dash_block_42(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    // ftCo_800DE9D8 (D-pad up taunt) needs a press: press_gate has stopped any.
    if relaxed_jump(p, rand)? {
        return Ok(());
    }
    if p.fp.cmd_vars[0] == 0 {
        return Ok(());
    }
    run_check_input(p, rand)?;
    Ok(())
}

/// The closing friction block of `ftCo_Dash_IASA`.
fn dash_friction(p: &mut Player) -> Result<(), String> {
    let cd = melee_fighter::hooks::common_data();
    let fr = fd_ground_friction(&p.fp);
    let fp = &mut *p.fp;
    let t = fp.gr_vel * cd.x54;
    fp.gr_vel = melee_core::fp::fmadds(-t, fr, fp.gr_vel);
    Ok(())
}

/// `ftCo_Dash_Phys`.
pub fn dash_phys(p: &mut Player) {
    if p.dash.x0 != 0.0 {
        p.dash.x0 = 0.0;
    } else {
        let cd = melee_fighter::hooks::common_data();
        let fp = &mut *p.fp;
        let (accel, target) = accel_and_target(fp);
        let fr = fp.co_attrs.ground_friction * cd.run_dash_turn_friction_multiplier;
        fc::calc_ground_accel_dash_run(fp, accel, target, fr);
    }
    fc::set_self_movement_from_grounded_movement(&mut p.fp);
}

/// `getAccelAndTarget`.
fn accel_and_target(fp: &Fighter) -> (f32, f32) {
    let x = fp.input.lstick[0].x;
    let mut accel = x * fp.co_attrs.dash_accel_mul;
    accel += if x > 0.0 { fp.co_attrs.dash_accel_base } else { -fp.co_attrs.dash_accel_base };
    (accel, x * fp.co_attrs.dash_max_velocity)
}

/// `ftCo_Run_Anim`.
pub fn run_anim(p: &mut Player) {
    let fp = &*p.fp;
    let vel = if fd_ground_friction(fp) < 1.0 { p.run.x4 } else { fp.gr_vel };
    let rate = if vel * fp.facing_dir <= 0.0 { 0.0 } else { fc::abs_c(vel) / fp.co_attrs.run_animation_scaling };
    set_anim_rate(p, rate);
    if p.run.x0 > 0.0 {
        p.run.x0 -= 1.0;
    }
}

/// `ftCo_Run_IASA`.
pub fn run_iasa(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    if special_checks(p, rand)? {
        return Ok(());
    }
    press_gate(p, "ftCo_Run_IASA")?;
    // ftCo_80091A4C (the shield), ftCo_800DE9D8: both need a press / trigger.
    if relaxed_jump(p, rand)? {
        return Ok(());
    }
    if p.run.x0 <= 0.0 {
        // fn_800C9D40: the stick back past x38 -> TurnRun.
        let cd = melee_fighter::hooks::common_data();
        let fp = &*p.fp;
        if fp.input.lstick[0].x * fp.facing_dir <= cd.x38_someLStickXThreshold {
            return Err(format!("player {}: ftCo_TurnRun_Enter not ported", p.slot + 1));
        }
        // ftCo_RunBrake_CheckInput: the stick under the run threshold.
        if fc::abs_c(fp.input.lstick[0].x) < cd.x58_someLStickXThreshold {
            return Err(format!("player {}: ftCo_RunBrake_Enter not ported", p.slot + 1));
        }
    }
    Ok(())
}

/// `ftCo_Run_Phys`.
pub fn run_phys(p: &mut Player) {
    let cd = melee_fighter::hooks::common_data();
    let fp = &mut *p.fp;
    let (mut accel, target) = accel_and_target(fp);
    if target != 0.0 {
        let gr_frac = fp.gr_vel / target;
        if gr_frac > 0.0 && gr_frac < 1.0 {
            accel *= (1.0 - gr_frac) * cd.run_accel_taper_gain;
        }
    }
    p.run.x4 = target * cd.x440;
    let fr = fp.co_attrs.ground_friction * cd.run_dash_turn_friction_multiplier;
    fc::calc_ground_accel_dash_run(fp, accel, target, fr);
    fc::set_self_movement_from_grounded_movement(fp);
}

/// `ftCo_Dash_Coll` / `ftCo_Run_Coll`: `ft_800844EC`.
pub fn coll(g: &mut GroundColl, cg: &mut CollGlobals, p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let joints = {
        let d = p.ecb_desc;
        ecb_joints(&mut p.skel, &d)
    };
    let fp = &mut *p.fp;
    fp.coll_data.last_pos = fp.coll_data.cur_pos;
    fp.coll_data.cur_pos = fp.cur_pos;
    let r = mc::mp_coll_8004b108(g, cg, &mut fp.coll_data, &joints).map_err(|e| format!("{} not ported", e.0))?;
    fp.cur_pos = fp.coll_data.cur_pos;
    if !r {
        return fall_enter(p, rand);
    }
    // ftCo_8009EDA4: running into a wall.
    let env = p.fp.coll_data.env_flags;
    let fd = p.fp.facing_dir;
    if ((fd == -1.0 && env & mc::COLLIDE_RIGHT_WALL_HUG != 0) || (fd == 1.0 && env & mc::COLLIDE_LEFT_WALL_HUG != 0))
        && fc::abs_c(p.fp.gr_vel) > p.fp.co_attrs.walk_max_vel
    {
        return Err(format!("player {}: ftCo_8009EE30 (StopWall) not ported", p.slot + 1));
    }
    Ok(())
}
