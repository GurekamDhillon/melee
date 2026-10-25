//! Samus' side special, the missile (`ftsamusspecials.c`, `ftsamusspecialn.c` `ftSs_SpecialS_8012A074`, and the
//! missile article `itsamusmissile.c`). The ground states only: the airborne ones return `Err`.

use super::*;

/// `ftSs_MS_SpecialS` .. `ftSs_MS_SpecialAirSSmash` (ftCo_MS_Count = 341, +8..+11) and their rows
/// (`ftSs_SM_SpecialS` = ftCo_SM_Count 295 + 8).
pub const SS_MS_SPECIAL_S: i32 = 349;
pub const SS_MS_SPECIAL_S_SMASH: i32 = 350;
pub const SS_MS_SPECIAL_AIR_S: i32 = 351;
pub const SS_MS_SPECIAL_AIR_S_SMASH: i32 = 352;
const SS_SM_SPECIAL_S: u32 = 303;

pub fn is_special_s(m: i32) -> bool {
    (SS_MS_SPECIAL_S..=SS_MS_SPECIAL_AIR_S_SMASH).contains(&m)
}

/// `ftSs_DatAttrs` x28 / x2C / x30 / x34 (`fp->dat_attrs`, `ft_data->x4`).
#[derive(Clone, Copy, Debug, Default)]
pub struct SsAttrs {
    pub x28: f32,
    pub x2c: f32,
    pub x30: f32,
    pub x34: f32,
}

impl SsAttrs {
    pub fn from_dat(d: &[u8], ft_data: u32) -> SsAttrs {
        let attr = be32(d, ft_data + 4);
        let f = |o: u32| f32::from_bits(be32(d, attr + o));
        SsAttrs { x28: f(0x28), x2c: f(0x2C), x30: f(0x30), x34: f(0x34) }
    }
}

/// Samus' own state: `accessory4_cb == ftSs_SpecialS_8012A074`.
#[derive(Clone, Copy, Debug, Default)]
pub struct SsVars {
    pub accessory4: bool,
}

/// `ftCo_SpecialS_CheckInput`'s `doEnter` and `ftSs_SpecialS_Enter` (on the ground).
pub fn special_s_enter(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let a = p.ss_attrs.ok_or("no Samus attrs")?;
    let cd = melee_fighter::hooks::common_data();
    let fp = &mut *p.fp;
    if fp.ground_or_air != melee_types::GA_Ground {
        return Err("ftSs_SpecialAirS_Enter is not ported".into());
    }
    if fp.input.lstick[0].x * fp.facing_dir < -cd.x220 {
        fc::update_facing(fp);
    }
    // doEnter: the ground speed retention.
    fp.gr_vel += -(fp.gr_vel * (1.0 - fp.co_attrs.specials_ground_speed_retention)) * melee_fighter::hooks::ground_friction_mul(fp);
    // ftSs_SpecialS_Enter.
    fp.gr_vel /= a.x2c;
    fp.self_vel.y = 0.0;
    let smash = (fp.active_sticky.lstick.x as f32) < a.x28;
    let (msid, smid) = if smash { (SS_MS_SPECIAL_S_SMASH, SS_SM_SPECIAL_S + 1) } else { (SS_MS_SPECIAL_S, SS_SM_SPECIAL_S) };
    let row = load_row(p, smid)?;
    change_motion_state(p, msid, row.as_ref(), rand)?;
    anim_step(p, rand)?;
    // ftSamus_ClearThrowFlagsUnk.
    unsafe { p.fp.anon_2210.throw_flags = 0 };
    p.ss.accessory4 = true;
    Ok(())
}

/// `ftSs_SpecialS_Anim` / `ftSs_SpecialSSmash_Anim`: Wait (or Fall in the air) when the animation ends.
pub fn anim(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    if p.fp.ground_or_air != melee_types::GA_Ground || matches!(p.fp.motion_id, SS_MS_SPECIAL_AIR_S | SS_MS_SPECIAL_AIR_S_SMASH) {
        return Err("Samus' airborne side special (anim) is not ported".into());
    }
    if !p.skel.frames_remaining() {
        ft_8008a2bc(p, rand)?;
    }
    Ok(())
}

/// `ftSs_SpecialS_Phys` (`ft_80084F3C`).
pub fn phys(p: &mut Player) -> Result<(), String> {
    if matches!(p.fp.motion_id, SS_MS_SPECIAL_AIR_S | SS_MS_SPECIAL_AIR_S_SMASH) {
        return Err("Samus' airborne side special (phys) is not ported".into());
    }
    let f = &mut *p.fp;
    let mut friction = f.co_attrs.ground_friction;
    if fc::abs_c(f.gr_vel) > f.co_attrs.walk_max_vel {
        friction *= melee_fighter::hooks::common_data().friction_when_above_walk_speed;
    }
    fc::calc_ground_accel_deaccel(f, friction);
    fc::set_self_movement_from_grounded_movement(f);
    Ok(())
}

/// `ftSs_SpecialS_Coll` (`ft_800827A0`: mpColl_8004B2DC, Fall when it leaves the floor) and
/// `ftSs_SpecialSSmash_Coll` (`ft_80082708`: mpColl_8004B108).
pub fn coll(g: &mut GroundColl, cg: &mut CollGlobals, p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let m = p.fp.motion_id;
    if matches!(m, SS_MS_SPECIAL_AIR_S | SS_MS_SPECIAL_AIR_S_SMASH) {
        return Err("Samus' airborne side special (coll) is not ported".into());
    }
    let joints = {
        let d = p.ecb_desc;
        ecb_joints(&mut p.skel, &d)
    };
    let fp = &mut *p.fp;
    fp.coll_data.last_pos = fp.coll_data.cur_pos;
    fp.coll_data.cur_pos = fp.cur_pos;
    let on = if m == SS_MS_SPECIAL_S {
        mc::mp_coll_8004b2dc(g, cg, &mut fp.coll_data, &joints)
    } else {
        mc::mp_coll_8004b108(g, cg, &mut fp.coll_data, &joints)
    }
    .map_err(|e| format!("{} not ported", e.0))?;
    fp.cur_pos = fp.coll_data.cur_pos;
    if !on {
        fall_enter(p, rand)?;
    }
    Ok(())
}

/// `ftSs_SpecialS_8012A074` (accessory4_cb, s_link 9): the missile at the throw flag.
pub fn accessory4(p: &mut Player) -> Result<(), String> {
    if !p.ss.accessory4 {
        return Ok(());
    }
    let fp = &mut *p.fp;
    if unsafe { fp.anon_2210.anon_0.throw_flags_b0() } == 0 {
        return Ok(());
    }
    // ftCheckThrowB0 clears the flag.
    unsafe { fp.anon_2210.anon_0.set_throw_flags_b0(0) };
    Err("ftSs_SpecialS_8012A074: the missile (it_802B62D0) is not ported".into())
}
