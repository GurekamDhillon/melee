//! Aerial attacks (ftCo_AttackAir.c): the common entry from the stick, the Anim / IASA / Phys /
//! Coll callbacks, and the landing out of an aerial. The landing-lag branch of
//! `ftCo_LandingAir_EnterWithLag` (`cmd_vars[0] != 0`, the aerial's L-cancel window) is not
//! ported: an aerial that lands before its window opens auto-cancels into plain Landing.

use super::*;

pub const FTCO_MS_ATTACK_AIR_N: i32 = 65;
pub const FTCO_MS_ATTACK_AIR_LW: i32 = 69;
/// `ftCo_SM_AttackAirN` (68).
const FTCO_SM_ATTACK_AIR_N: u32 = 68;

pub fn is_attack_air(m: i32) -> bool {
    (FTCO_MS_ATTACK_AIR_N..=FTCO_MS_ATTACK_AIR_LW).contains(&m)
}

/// `ftCo_800DF478`: a C-stick flick past the thresholds this frame.
pub fn cstick_flick(fp: &Fighter) -> bool {
    let cd = melee_fighter::hooks::common_data();
    let i = &fp.input;
    (fc::abs_c(i.cstick[1].x) < cd.xDC && fc::abs_c(i.cstick[0].x) >= cd.xDC) || (fc::abs_c(i.cstick[1].y) < cd.xE0 && fc::abs_c(i.cstick[0].y) >= cd.xE0)
}

/// `ftCo_AttackAir_GetMsidFromCStick`.
fn msid_from_stick(fp: &Fighter) -> i32 {
    let cd = melee_fighter::hooks::common_data();
    let i = &fp.input;
    let (sx, sy) = if cstick_flick(fp) { (i.cstick[0].x, i.cstick[0].y) } else { (i.lstick[0].x, i.lstick[0].y) };
    let angle = melee_runtime::lbtrigf::atan2f(sy, fc::abs_c(sx));
    if fc::abs_c(sx) < cd.xDC && fc::abs_c(sy) < cd.xE0 {
        return FTCO_MS_ATTACK_AIR_N;
    }
    if angle > cd.x20_radians {
        return FTCO_MS_ATTACK_AIR_N + 3;
    }
    if angle < -cd.x20_radians {
        return FTCO_MS_ATTACK_AIR_N + 4;
    }
    if sx * fp.facing_dir >= 0.0 {
        FTCO_MS_ATTACK_AIR_N + 1
    } else {
        FTCO_MS_ATTACK_AIR_N + 2
    }
}

/// `ftCo_AttackAir_CheckItemThrowInput` (no item): A or a C-stick flick starts the aerial.
pub fn check_input(p: &mut Player, rand: &mut HsdRand) -> Result<bool, String> {
    let fp = &*p.fp;
    if fp.input.pressed_buttons & input::HSD_PAD_A == 0 && !cstick_flick(fp) {
        return Ok(false);
    }
    if !fp.item_gobj.is_null() {
        return Err(format!("player {}: ftCo_AttackAir_CheckItemThrowInput with an item not ported", p.slot + 1));
    }
    match fp.kind {
        melee_types::Ft_Kind_Link | melee_types::Ft_Kind_CLink | melee_types::Ft_Kind_GameWatch => {
            return Err(format!("player {}: character-specific aerial (Link / G&W) not ported", p.slot + 1));
        }
        _ => {}
    }
    let msid = msid_from_stick(fp);
    enter_from_msid(p, msid, rand)?;
    Ok(true)
}

/// `ftCo_AttackAir_EnterFromMsid`.
fn enter_from_msid(p: &mut Player, msid: i32, rand: &mut HsdRand) -> Result<(), String> {
    p.fp.set_allow_interrupt(0);
    p.fp.cmd_vars[0] = 0;
    p.fp.anon_2210.throw_flags = 0;
    let row = load_row(p, FTCO_SM_ATTACK_AIR_N + (msid - FTCO_MS_ATTACK_AIR_N) as u32)?;
    change_motion_state(p, msid, row.as_ref(), rand)?;
    anim_step(p, rand) // ftAnim_8006EBA4
}

/// `ftCo_AttackAir_Anim`.
pub fn anim(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    if unsafe { p.fp.anon_2210.throw_flags } != 0 {
        return Err(format!("player {}: aerial throw_flags (ftCheckThrowB3) not ported", p.slot + 1));
    }
    if !p.skel.frames_remaining() {
        fall_enter(p, rand)?;
    }
    Ok(())
}

/// `ftCo_AttackAir*_IASA` (DO_IASA): after the interrupt window opens, another aerial or a
/// double jump.
pub fn iasa(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    if p.fp.allow_interrupt() == 0 {
        return Ok(());
    }
    if !p.fp.item_gobj.is_null() {
        return Err("ftCo_AttackAir_IASA with an item not ported".into());
    }
    {
        let fp = &*p.fp;
        let i = &fp.input;
        if i.pressed_buttons & !(PAD_STICK_DIRS | input::HSD_PAD_XY | input::HSD_PAD_A) != 0 || i.triggers[0] != 0.0 {
            return Err(format!("player {}: ftCo_AttackAir_IASA with a press / trigger: not ported", p.slot + 1));
        }
    }
    if check_input(p, rand)? {
        return Ok(());
    }
    ftco_800cb870(p, rand).map(|_| ())
}

/// `ftCo_AttackAir_Phys`: `ft_80084DB0`.
pub fn phys(p: &mut Player) {
    let f = &mut *p.fp;
    fc::check_fall_fast(f);
    if f.fall_fast() != 0 {
        fc::fall_fast(f);
    } else {
        let (g, t) = (f.co_attrs.gravity, f.co_attrs.terminal_velocity);
        fc::fall(f, g, t);
    }
    fc::calc_self_accel_drift(f);
}

/// `ftCo_AttackAir_Coll`: `ft_80082C74(gobj, ftCo_LandingAir_EnterWithLag)`.
pub fn coll(g: &mut GroundColl, cg: &mut CollGlobals, p: &mut Player, c: EntryCommon, rand: &mut HsdRand) -> Result<(), String> {
    let joints = {
        let d = p.ecb_desc;
        ecb_joints(&mut p.skel, &d)
    };
    let fp = &mut *p.fp;
    // ft_80081D0C: mpColl_800471F8.
    fp.coll_data.last_pos = fp.coll_data.cur_pos;
    fp.coll_data.cur_pos = fp.cur_pos;
    let r = mc::mp_coll_800471f8(g, cg, &mut fp.coll_data, &joints).map_err(|e| format!("{} not ported", e.0))?;
    fp.cur_pos = fp.coll_data.cur_pos;
    if ft_80081a00(fp) || !r {
        return Ok(());
    }
    // ftCo_LandingAir_EnterWithLag with cmd_vars[0] == 0: ftCo_Landing_Enter_Basic.
    if p.fp.cmd_vars[0] != 0 {
        return Err(format!("player {}: ftCo_LandingAir_EnterWithLag (L-cancel window) not ported", p.slot + 1));
    }
    landing_enter_basic(g, p, c, rand)
}
