//! Fox's and Falco's Reflector (down special; `ftfoxspeciallw.c`): Start / Loop / Hit / End, on the ground
//! and in the air. The Turn states (stick flick during the loop) are refused.
//!
//! Not modelled: the reflect capsule's bone (`reflect_hit.bone`; only read by hit detection against a
//! projectile), `Fighter_SetEffectHitlagCallbacks` (effects pause in hitlag), the pass-through platform
//! drop (`ftFx_SpecialLwStart_Pass`).

use super::*;

pub const LW_START: i32 = 360;
pub const LW_LOOP: i32 = 361;
pub const LW_HIT: i32 = 362;
pub const LW_END: i32 = 363;
pub const LW_TURN: i32 = 364;
pub const AIR_LW_START: i32 = 365;
pub const AIR_LW_LOOP: i32 = 366;
pub const AIR_LW_HIT: i32 = 367;
pub const AIR_LW_END: i32 = 368;
pub const AIR_LW_TURN: i32 = 369;

/// `ftFx_SM_SpecialLwStart` (313) .. `SpecialAirLwEnd` (320); the Turn states play the Loop rows.
fn smid(ms: i32) -> u32 {
    match ms {
        LW_START => 313,
        LW_LOOP | LW_TURN => 314,
        LW_HIT => 315,
        LW_END => 316,
        AIR_LW_START => 317,
        AIR_LW_LOOP | AIR_LW_TURN => 318,
        AIR_LW_HIT => 319,
        _ => 320,
    }
}

pub fn is_lw(m: i32) -> bool {
    (LW_START..=AIR_LW_TURN).contains(&m)
}

fn is_air(m: i32) -> bool {
    m >= AIR_LW_START
}

/// `ftCommon_GroundAirColl_MF | Ft_MF_KeepGfx` (`ftFx_MF_SpecialLw_Coll`).
const MF_COLL: u32 = (melee_types::Ft_MF_SkipMatAnim
    | melee_types::Ft_MF_SkipColAnim
    | melee_types::Ft_MF_UpdateCmd
    | melee_types::Ft_MF_SkipItemVis
    | melee_types::Ft_MF_Unk19
    | melee_types::Ft_MF_SkipModelPartVis
    | melee_types::Ft_MF_SkipModelFlags
    | melee_types::Ft_MF_Unk27
    | melee_types::Ft_MF_KeepGfx) as u32;
/// `ftFx_MF_SpecialLwEnd_Coll`.
const MF_END_COLL: u32 = (melee_types::Ft_MF_SkipColAnim | melee_types::Ft_MF_UpdateCmd) as u32;

fn attrs(p: &Player) -> Result<melee_types::ftFox_DatAttrs, String> {
    p.fx_attrs.ok_or_else(|| "no Fox attrs".to_string())
}

/// `ftFox_SpecialLw_SetVars`.
fn set_vars(p: &mut Player) -> Result<(), String> {
    let da = attrs(p)?;
    p.fx.lw_release_lag = melee_runtime::cabi::f2i32(da.x98_FOX_REFLECTOR_RELEASE_LAG as f64);
    p.fx.lw_release = false;
    p.fp.cmd_vars[1] = 4;
    p.fx.lw_gravity_delay = da.xA4_FOX_REFLECTOR_GRAVITY_DELAY;
    p.fx.lw_gfx = 1;
    Ok(())
}

/// `ftFx_SpecialLw_Enter` / `ftFx_SpecialAirLw_Enter`.
pub fn enter(p: &mut Player, air: bool, rand: &mut HsdRand) -> Result<(), String> {
    let da = attrs(p)?;
    if air {
        p.fp.self_vel.y = 0.0;
        p.fp.self_vel.x /= da.xA8_FOX_REFLECTOR_MOMENTUM_PRESERVE_X;
    }
    let ms = if air { AIR_LW_START } else { LW_START };
    let row = load_row(p, smid(ms))?;
    change_motion_state(p, ms, row.as_ref(), rand)?;
    anim_step(p, rand)?;
    set_vars(p)
}

/// `ftColl_CreateReflectHit(gobj, &da->xB0_FOX_REFLECTOR_REFLECTION, ftFx_SpecialLwHit_Enter)`.
fn create_reflect_hit(p: &mut Player) -> Result<(), String> {
    let da = attrs(p)?;
    let r = da.xB0_FOX_REFLECTOR_REFLECTION;
    let fp = &mut *p.fp;
    fp.set_reflecting(1);
    fp.set_x2218_b4(0);
    p.fx.lw_reflect = true;
    fp.ReflectAttr.x1A30_maxDamage = r.x4_max_damage;
    fp.ReflectAttr.x1A34_damageMul = r.x18_damage_mul;
    fp.ReflectAttr.x1A38_speedMul = r.x1C_speed_mul;
    fp.set_x2218_b5(r.x20_behavior);
    fp.reflect_hit.size = r.x14_size;
    fp.reflect_hit.offset = r.x8_offset;
    Ok(())
}

/// `ftFx_SpecialLwHit_SetCall`.
fn set_call(p: &mut Player) {
    p.fp.set_reflecting(1);
    p.fx.lw_reflect = true;
}

/// `ftFx_SpecialLwLoop_Enter` / `ftFx_SpecialAirLwLoop_Enter`: Loop with Ft_MF_KeepGfx, then the reflect capsule.
fn loop_enter(p: &mut Player, air: bool, rand: &mut HsdRand) -> Result<(), String> {
    let ms = if air { AIR_LW_LOOP } else { LW_LOOP };
    let row = load_row(p, smid(ms))?;
    p.keep_gfx = true;
    change_motion_state(p, ms, row.as_ref(), rand)?;
    create_reflect_hit(p)
}

/// `ftFx_SpecialLwEnd_Enter` / `ftFx_SpecialAirLwEnd_Enter`.
fn end_enter(p: &mut Player, air: bool, rand: &mut HsdRand) -> Result<(), String> {
    let ms = if air { AIR_LW_END } else { LW_END };
    let row = load_row(p, smid(ms))?;
    change_motion_state(p, ms, row.as_ref(), rand)
}

/// The three-line prologue every Anim callback shares: B released, the release lag counting down (not in Start).
fn release_tick(p: &mut Player, lag: bool) {
    if p.fp.input.held_buttons[0] & input::HSD_PAD_B == 0 {
        p.fx.lw_release = true;
    }
    if lag && p.fx.lw_release_lag > 0 {
        p.fx.lw_release_lag -= 1;
    }
}

/// `ftFx_SpecialLwHit_Check`: leave for End once released, else back into the Loop.
fn hit_check(p: &mut Player, rand: &mut HsdRand) -> Result<bool, String> {
    let air = p.fp.ground_or_air != melee_types::GA_Ground;
    if p.fx.lw_release_lag <= 0 && p.fx.lw_release {
        end_enter(p, air, rand)?;
        return Ok(false);
    }
    let ms = if air { AIR_LW_LOOP } else { LW_LOOP };
    let row = load_row(p, smid(ms))?;
    p.keep_gfx = true;
    change_motion_state(p, ms, row.as_ref(), rand)?;
    create_reflect_hit(p)?;
    Ok(true)
}

/// The Anim callbacks.
pub fn anim(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let m = p.fp.motion_id;
    let air_gr = p.fp.ground_or_air != melee_types::GA_Ground;
    match m {
        LW_START | AIR_LW_START => {
            release_tick(p, false);
            if !p.skel.frames_remaining() {
                loop_enter(p, air_gr, rand)?;
                ftcommon_8007db24(p);
                p.fx.lw_gfx = 2;
            }
        }
        LW_LOOP | AIR_LW_LOOP => {
            release_tick(p, true);
            if p.fx.lw_release_lag <= 0 && p.fx.lw_release {
                end_enter(p, air_gr, rand)?;
            }
        }
        LW_HIT | AIR_LW_HIT => {
            release_tick(p, true);
            if !p.skel.frames_remaining() && hit_check(p, rand)? {
                ftcommon_8007db24(p);
                p.fx.lw_gfx = 2;
            }
        }
        LW_END | AIR_LW_END => {
            if !p.skel.frames_remaining() {
                ftcommon_8007db24(p);
                // ftCommon_8007D92C.
                if p.fp.ground_or_air == melee_types::GA_Air {
                    fall_enter(p, rand)?;
                } else {
                    ft_8008a2bc(p, rand)?;
                }
            }
        }
        _ => return Err(format!("player {}: Fox Reflector Turn ({m}) not ported", p.slot + 1)),
    }
    Ok(())
}

/// `ftCo_800C97A8`: the stick pulled back (a Turn).
fn turn_check(p: &Player) -> bool {
    p.fp.input.lstick[0].x * p.fp.facing_dir <= melee_fighter::hooks::common_data().x34
}

/// `ftCo_80099F1C`: stick down on a pass-through platform.
fn pass_check(p: &Player) -> bool {
    let cd = melee_fighter::hooks::common_data();
    let fp = &*p.fp;
    fp.input.lstick[0].y <= -cd.x464 && (fp.active_timer.lstick.y as i32) < cd.x468 as i32 && p.on_platform
}

/// The IASA callbacks (only Start and the Loops have any).
pub fn iasa(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let m = p.fp.motion_id;
    match m {
        LW_START => {
            if pass_check(p) {
                return Err(format!("player {}: ftFx_SpecialLwStart_Pass not ported", p.slot + 1));
            }
        }
        LW_LOOP => {
            if turn_check(p) {
                return Err(format!("player {}: ftFx_SpecialLwTurn (Reflector turn) not ported", p.slot + 1));
            }
            if jump_check_input(p, rand)? {
                return Ok(());
            }
            if pass_check(p) {
                return Err(format!("player {}: ftFx_SpecialLwLoop_Pass not ported", p.slot + 1));
            }
        }
        AIR_LW_LOOP => {
            if turn_check(p) {
                return Err(format!("player {}: ftFx_SpecialAirLwTurn (Reflector turn) not ported", p.slot + 1));
            }
            ftco_800cb870(p, rand)?;
        }
        _ => {}
    }
    Ok(())
}

/// The Phys callbacks. `friction` is `ft_80084F3C`'s ground friction; the air halves fall after `gravityDelay`.
pub fn phys(p: &mut Player) -> Result<(), String> {
    let m = p.fp.motion_id;
    if !is_air(m) {
        // ft_80084F3C (+ ftColl_8007AEF8, the reflect capsule's position cache, for Loop / Hit).
        let f = &mut *p.fp;
        let mut friction = f.co_attrs.ground_friction;
        if fc::abs_c(f.gr_vel) > f.co_attrs.walk_max_vel {
            friction *= melee_fighter::hooks::common_data().friction_when_above_walk_speed;
        }
        fc::calc_ground_accel_deaccel(f, friction);
        fc::set_self_movement_from_grounded_movement(f);
        return Ok(());
    }
    let da = attrs(p)?;
    if p.fx.lw_gravity_delay != 0 {
        p.fx.lw_gravity_delay -= 1;
    } else {
        let f = &mut *p.fp;
        let t = f.co_attrs.terminal_velocity;
        fc::fall(f, da.xAC_FOX_REFLECTOR_FALL_ACCEL, t);
    }
    fc::calc_self_accel_deaccel_quick_air(&mut p.fp);
    Ok(())
}

/// The Coll callbacks: ground states walk off the floor end (-> the air state at the same frame), air
/// states land (-> the ground state).
pub fn coll(g: &mut GroundColl, cg: &mut CollGlobals, p: &mut Player, c: EntryCommon, rand: &mut HsdRand) -> Result<(), String> {
    let m = p.fp.motion_id;
    let joints = {
        let d = p.ecb_desc;
        ecb_joints(&mut p.skel, &d)
    };
    let fp = &mut *p.fp;
    fp.coll_data.last_pos = fp.coll_data.cur_pos;
    fp.coll_data.cur_pos = fp.cur_pos;
    if !is_air(m) {
        let r = mc::mp_coll_8004b108(g, cg, &mut fp.coll_data, &joints).map_err(|e| format!("{} not ported", e.0))?;
        fp.cur_pos = fp.coll_data.cur_pos;
        if r {
            return Ok(());
        }
        // GroundToAir: ftCommon_8007D5D4, then the air state at the same animation frame.
        if m == LW_TURN {
            return Err("Reflector Turn off the floor not ported".into());
        }
        melee_fighter::ftcommon2::ftcommon_8007d5d4(&mut p.fp);
        let to = m + 5;
        let flags = if m == LW_END { MF_END_COLL } else { MF_COLL };
        let row = load_row(p, smid(to))?;
        let frame = p.fp.cur_anim_frame;
        change_motion_state_flags(p, to, row.as_ref(), frame, 1.0, flags, rand)?;
        match m {
            LW_LOOP => create_reflect_hit(p)?,
            LW_HIT => set_call(p),
            _ => {}
        }
        return Ok(());
    }
    let r = mc::mp_coll_800471f8(g, cg, &mut fp.coll_data, &joints).map_err(|e| format!("{} not ported", e.0))?;
    fp.cur_pos = fp.coll_data.cur_pos;
    if !r {
        return Ok(());
    }
    if m == AIR_LW_TURN {
        return Err("Reflector Turn landing not ported".into());
    }
    // AirToGround: ftCommon_8007D7FC, the ground state at the same frame, ClampAirDrift, then the reflect capsule.
    ftcommon_8007d7fc(g, p, c)?;
    let to = m - 5;
    let flags = if m == AIR_LW_END { MF_END_COLL } else { MF_COLL };
    let row = load_row(p, smid(to))?;
    let frame = p.fp.cur_anim_frame;
    change_motion_state_flags(p, to, row.as_ref(), frame, 1.0, flags, rand)?;
    fc::clamp_air_drift(&mut p.fp);
    match m {
        AIR_LW_LOOP => create_reflect_hit(p)?,
        AIR_LW_HIT => set_call(p),
        _ => {}
    }
    Ok(())
}

impl VsMatch {
    /// `ftFx_SpecialLw_Create{Start,Loop,Reflect}GFX` (accessory4_cb, s_link 9): the bubble effect on HipN.
    pub(super) fn fox_lw_accessory4(&mut self, i: usize) -> Result<(), String> {
        let k = std::mem::take(&mut self.players[i].fx.lw_gfx);
        if k == 0 {
            return Ok(());
        }
        if self.players[i].fp.x2219_b0() == 0 {
            // efSync_Spawn(1160 / 1161 / 1162) -> efLib_Create_Attach_Scale(0xBB8 / 0xBB9 / 0xBBA, gobj, HipN).
            let scale = self.players[i].model_scaling;
            let id = match k {
                1 => 0xBB9,
                2 => 0xBB8,
                _ => 0xBBA,
            };
            self.ef_create(id, i, scale, None, None)?;
            self.players[i].fp.set_x2219_b0(1);
        }
        Ok(())
    }
}
