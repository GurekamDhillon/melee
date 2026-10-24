//! Sheik's neutral special (Needle Storm, ftseakspecialn.c) and the two items it makes
//! (itseakneedleheld.c, itseakneedlethrown.c).
//!
//! Items are the hashed fields only (kind, msid, pos, vel: docs/rust-port/neutral-hash.md) plus
//! the little state the item procs need. A thrown needle is modelled in its flying state
//! (msid 0, `itSeakneedlethrown_UnkMotion0_*`); a ground hit (the stick / bounce states 1..4) and
//! a hit on a fighter stop the run. Not modelled: SFX, the held bundle's model (`it_802B18B0`),
//! the damage callback (`ftSk_Init_80110198`, only a hit reaches it) and the colour overlay of a
//! full charge (`ftCo_800BFFD0`, render only).

use super::fox::{ItemClass, ItemCmd, SimItem};
use super::*;

pub const SK_MS_SPECIAL_N_START: i32 = 341;
pub const SK_MS_SPECIAL_N_LOOP: i32 = 342;
pub const SK_MS_SPECIAL_N_CANCEL: i32 = 343;
pub const SK_MS_SPECIAL_N_END: i32 = 344;
pub const SK_MS_SPECIAL_AIR_N_START: i32 = 345;
pub const SK_MS_SPECIAL_AIR_N_LOOP: i32 = 346;
pub const SK_MS_SPECIAL_AIR_N_CANCEL: i32 = 347;
pub const SK_MS_SPECIAL_AIR_N_END: i32 = 348;
/// `ftSk_SM_SpecialNStart` = `ftCo_SM_Count`; the subaction order is the motion order.
const SK_SM_SPECIAL_N_START: u32 = 295;

/// The thrown needle's article (ItCo's `itPublicData` -> character items, kind 79): the common
/// attributes' `destroy_gfx` / `destroy_sfx` and the special attributes' lifetime (`x0`), the
/// post-hit lifetime (`x4`) and speed (`x8`).
#[derive(Clone, Copy, Debug)]
pub struct NeedleAttrs {
    pub destroy_gfx: i32,
    pub life: f32,
    /// `x4`: the lifetime once it sticks in the ground or bounces.
    pub hit_life: f32,
    pub speed: f32,
}

impl NeedleAttrs {
    /// `ft_data->x48_items[0]` (registered as `It_Kind_Seak_NeedleThrow`) in PlSk.dat.
    pub fn from_dat(data: &[u8], ft_data: u32) -> Self {
        let be = |at: u32| u32::from_be_bytes(data[at as usize..][..4].try_into().unwrap());
        let art = be(be(ft_data + 0x48));
        let (common, special) = (be(art), be(art + 4));
        NeedleAttrs { destroy_gfx: be(common + 0x64) as i32, life: f32::from_bits(be(special)), hit_life: f32::from_bits(be(special + 4)), speed: f32::from_bits(be(special + 8)) }
    }
}

/// `FtPart` of the held bundle (`it_802B19AC(gobj, &cur_pos, 23, ...)`): not modelled.
/// The needle's lifetime (`itSeakNeedleThrownAttributes.x0`) and speed (`x8`), checked against
/// the golden (30 frames; |vel| = 4).
/// `needleYPosScale`.
const NEEDLE_Y_POS_SCALE: [f32; 9] = [-1.0, -0.75, -0.5, -0.25, 0.0, 0.25, 0.5, 0.75, 1.0];

pub fn is_sheik_special_n(m: i32) -> bool {
    (SK_MS_SPECIAL_N_START..=SK_MS_SPECIAL_AIR_N_END).contains(&m)
}

fn is_air(m: i32) -> bool {
    m >= SK_MS_SPECIAL_AIR_N_START
}

/// `fp->u.sk` (x0 = the charge count, x4 = the held bundle) and `fp->mv.sk.specialn`, plus
/// `fp->accessory4_cb == shootNeedles`.
#[derive(Clone, Copy, Debug, Default)]
pub struct SeakVars {
    pub count: i32,
    pub held: bool,
    pub n0: i32,
    pub n4: bool,
    pub n8: i32,
    pub accessory4: bool,
}

/// `doEnter`: `ftSk_SpecialN_Enter` / `ftSk_SpecialAirN_Enter`.
pub fn special_n_enter(p: &mut Player, air: bool, rand: &mut HsdRand) -> Result<(), String> {
    let msid = if air { SK_MS_SPECIAL_AIR_N_START } else { SK_MS_SPECIAL_N_START };
    let row = load_row(p, SK_SM_SPECIAL_N_START + (msid - SK_MS_SPECIAL_N_START) as u32)?;
    change_motion_state(p, msid, row.as_ref(), rand)?;
    // fp->throw_flags_b0 = false; Fighter_ClearCmdVars.
    unsafe { p.fp.anon_2210.anon_0.set_throw_flags_b0(0) };
    p.fp.cmd_vars = [0; 4];
    let sk = &mut p.sk;
    sk.n0 = 0;
    if sk.count == 0 {
        sk.count = 1;
    }
    sk.n4 = false;
    sk.n8 = 0;
    sk.accessory4 = false;
    anim_step(p, rand)?; // ftAnim_8006EBA4
    Ok(())
}

/// `da->x10` of the landing-lag branches: `ftCo_Fall_Enter` at 0, else `ftCo_80096900`.
fn end_to_fall(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let da = p.sk_attrs.ok_or("no Sheik attrs")?;
    if da.x10 == 0.0 {
        fall_enter(p, rand)
    } else {
        Err("ftCo_80096900 (Needle Storm landing lag) not ported".into())
    }
}

/// The Anim callbacks of the eight states.
pub fn anim(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let m = p.fp.motion_id;
    let air = is_air(m);
    let state = (m - SK_MS_SPECIAL_N_START) & 3;
    let base = if air { SK_MS_SPECIAL_AIR_N_START } else { SK_MS_SPECIAL_N_START };
    match state {
        0 => {
            // Start: the bundle appears when the animation ends, then Loop.
            if !p.skel.frames_remaining() {
                let pos = p.fp.cur_pos;
                p.item_cmds.push(ItemCmd::SeakHeld { owner: p.slot as usize, pos: Vec3 { x: pos.x, y: pos.y, z: 0.0 } });
                p.sk.held = true;
                let row = load_row(p, SK_SM_SPECIAL_N_START + (base + 1 - SK_MS_SPECIAL_N_START) as u32)?;
                change_motion_state(p, base + 1, row.as_ref(), rand)?;
            }
        }
        1 => {
            // Loop: ftSk_SpecialNLoop_Anim (Sheik_ChargeNeedlesIncrementer).
            let sk = &mut p.sk;
            sk.n8 += 1;
            if p.fp.cur_anim_frame == 0.0 {
                sk.count += 1;
                sk.n8 = 0;
                if sk.count > 6 {
                    sk.count = 6;
                    sk.n8 = 100;
                    // ftCo_800BFFD0(fp, 86, 0): the colour overlay, render only.
                }
            }
        }
        2 => {
            // Cancel.
            p.sk.held = false;
            if !p.skel.frames_remaining() {
                if air {
                    end_to_fall(p, rand)?;
                } else {
                    ft_8008a2bc(p, rand)?;
                }
            }
        }
        _ => {
            // End: a needle every third frame from the second.
            let sk = &mut p.sk;
            if matches!(sk.n0, 2 | 5 | 8 | 11 | 14 | 17) {
                sk.n4 = true;
                sk.held = false;
            }
            sk.n0 += 1;
            if !p.skel.frames_remaining() {
                if air {
                    end_to_fall(p, rand)?;
                } else {
                    ft_8008a2bc(p, rand)?;
                }
            }
        }
    }
    Ok(())
}

/// The IASAs (`doIasa` for the Loop states; the others are empty).
pub fn iasa(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let m = p.fp.motion_id;
    if !matches!(m, SK_MS_SPECIAL_N_LOOP | SK_MS_SPECIAL_AIR_N_LOOP) {
        return Ok(());
    }
    let air = m == SK_MS_SPECIAL_AIR_N_LOOP;
    let base = if air { SK_MS_SPECIAL_AIR_N_START } else { SK_MS_SPECIAL_N_START };
    if p.fp.input.held_buttons[0] & input::HSD_PAD_B == 0 {
        p.sk.n0 = 0;
        let end = base + 3;
        let row = load_row(p, SK_SM_SPECIAL_N_START + (end - SK_MS_SPECIAL_N_START) as u32)?;
        change_motion_state(p, end, row.as_ref(), rand)?;
        p.sk.accessory4 = true;
    } else if p.fp.input.pressed_buttons & input::HSD_PAD_LR != 0 {
        let cancel = base + 2;
        let row = load_row(p, SK_SM_SPECIAL_N_START + (cancel - SK_MS_SPECIAL_N_START) as u32)?;
        change_motion_state(p, cancel, row.as_ref(), rand)?;
    }
    Ok(())
}

/// The Phys callbacks: `ft_80084F3C` (ground) / `ft_80084EEC` (air).
pub fn phys(p: &mut Player) {
    let f = &mut *p.fp;
    if is_air(f.motion_id) {
        let (g, t) = (f.co_attrs.gravity, f.co_attrs.terminal_velocity);
        fc::fall(f, g, t);
        let fr = f.co_attrs.aerial_friction;
        fc::calc_self_accel_deaccel(f, fr);
    } else {
        let mut friction = f.co_attrs.ground_friction;
        if fc::abs_c(f.gr_vel) > f.co_attrs.walk_max_vel {
            friction *= melee_fighter::hooks::common_data().friction_when_above_walk_speed;
        }
        fc::calc_ground_accel_deaccel(f, friction);
        fc::set_self_movement_from_grounded_movement(f);
    }
}

/// `shootNeedles` (accessory4_cb, s_link 9).
impl VsMatch {
    pub(super) fn seak_accessory(&mut self, i: usize) -> Result<(), String> {
        let p = &self.players[i];
        if p.fp.kind != melee_types::Ft_Kind_Seak || !p.sk.accessory4 || !matches!(p.fp.motion_id, SK_MS_SPECIAL_N_END | SK_MS_SPECIAL_AIR_N_END) {
            return Ok(());
        }
        if !p.sk.n4 {
            return Ok(());
        }
        let da = p.sk_attrs.ok_or("no Sheik attrs")?;
        let na = p.sk_needle.ok_or("no Sheik needle article")?;
        self.players[i].sk.n4 = false;
        if self.players[i].sk.count <= 0 {
            return Ok(());
        }
        let fp = &self.players[i].fp;
        let (scale_y, facing, air) = (fp.x34_scale.y, fp.facing_dir, fp.ground_or_air != melee_types::GA_Ground);
        let mut pos = fp.cur_pos;
        let mid = 0.5f32 * (fp.coll_data.ecb.top.y + fp.coll_data.ecb.bottom.y);
        let parent = Vec3 { x: fp.cur_pos.x + 0.0, y: fp.cur_pos.y + mid, z: fp.cur_pos.z + 0.0 };
        let r = self.rand.randi(9) as usize;
        if !air {
            pos.x += scale_y * (da.x0 * facing);
            pos.y += scale_y * (da.x4 + NEEDLE_Y_POS_SCALE[r]);
        } else {
            pos.x += scale_y * (da.x8 * facing);
            pos.y += scale_y * ((2.0f32 * NEEDLE_Y_POS_SCALE[r]) + da.xC);
        }
        pos.z = 0.0;
        // it_802AFD8C -> Item_80268B18 (the spawn step from the parent to `pos`), it_802AFEA8(.., 0)
        // -> it_802AFF08: the launch velocity.
        let at = super::fox::spawn_step(parent, Vec3 { x: pos.x, y: pos.y, z: 0.0 });
        let angle_factor: f32 = if air { 2.3561945 } else { 1.5707964 };
        let a = melee_core::fp::fmadds(facing, angle_factor, 1.5707964);
        let vel = Vec3 {
            x: -na.speed * melee_runtime::gekko::cosf(a),
            y: na.speed * melee_runtime::gekko::sinf(a),
            z: 0.0,
        };
        self.items.push(SimItem {
            kind: melee_types::It_Kind_Seak_NeedleThrow as i32,
            msid: 0,
            pos: at,
            vel,
            class: ItemClass::SeakThrown { owner: i, life: na.life, fresh: true, state: 0, line: -1, bx: 0.0, bmin: 0.0, bgrav: 0.0 },
        });
        self.players[i].sk.count -= 1;
        // efSync_Spawn(1283, gobj, &pos) = efLib_CreateGenerator(0x6A, &pos).
        let mut host = Host { rand: &mut self.rand, joint_mtx: &self.joint_mtx, appsrt_hook_lists: &self.appsrt_hook_lists };
        if let Some(gen) = self.ps.gen_create(&mut host, 0, 0, 0x6A) {
            self.ps.gen_mut(gen).pos = Vec3 { x: pos.x, y: pos.y, z: pos.z };
        }
        Ok(())
    }
}

/// The item procs of the two needle items. `Some(dead)` for a Sheik item, `None` otherwise.
pub(super) fn item_proc(m: &mut VsMatch, it: &mut SimItem) -> Result<Option<bool>, String> {
    match &mut it.class {
        ItemClass::SeakHeld { owner } => {
            // itSeakneedleheld_UnkMotion0_Anim: gone once the owner's u.sk.x4 is clear.
            Ok(Some(!m.players[*owner].sk.held))
        }
        ItemClass::SeakThrown { owner, life, fresh, state, line, bx, bmin, bgrav } => {
            if *fresh {
                *fresh = false;
                return Ok(Some(false));
            }
            let owner = *owner;
            let na = m.players[owner].sk_needle.ok_or("no Sheik needle article")?;
            // Anim: xDE4 = pos; the flying / stuck states count the life down (it_80273130).
            let prev = it.pos;
            if *state != 4 {
                *life -= 1.0;
                if *life <= 0.0 {
                    // Item_8026A8EC -> ItemSwitch -> it_8027327C(destroy_gfx = 0x421) -> it_80278800:
                    // three HSD_Randf for the (zero-sized) random offset, then efSync_Spawn(0x421)
                    // -> efLib_CreateGenerator(0x3F, item position).
                    for _ in 0..3 {
                        m.rand.randf();
                    }
                    let mut host = Host { rand: &mut m.rand, joint_mtx: &m.joint_mtx, appsrt_hook_lists: &m.appsrt_hook_lists };
                    if let Some(gen) = m.ps.gen_create(&mut host, 0, 0, 0x3F) {
                        m.ps.gen_mut(gen).pos = Vec3 { x: it.pos.x, y: it.pos.y, z: it.pos.z };
                    }
                    return Ok(Some(true));
                }
            }
            // Phys: state 4 falls (UnkMotion4_Phys).
            if *state == 4 {
                it.vel.x = *bx;
                it.vel.y += *bgrav;
                if it.vel.y < *bmin {
                    it.vel.y = *bmin;
                }
            }
            // Item_802697D4: pos += vel (the nudge and the floor speed are zero in the air).
            it.pos.x += it.vel.x;
            it.pos.y += it.vel.y;
            it.pos.z += it.vel.z;
            match *state {
                0 | 4 => {
                    // CheckGroundHit: it_8026EA20(prev -> pos) against the stage lines, then
                    // mpGetSpeed(line) as the velocity.
                    let hit = m.ground.mp_check_all_remap(-1, -1, prev.x, prev.y, it.pos.x, it.pos.y);
                    if let Some(h) = hit.filter(|h| m.ground.mp_lib_80054ed8(h.line_id)) {
                        it.pos = Vec3 { x: h.pos.x, y: h.pos.y, z: h.pos.z };
                        *line = h.line_id;
                        let d = m.ground.mp_get_speed(h.line_id, melee_world::vec::Vec3 { x: it.pos.x, y: it.pos.y, z: it.pos.z });
                        it.vel = d.map_or(Vec3 { x: 0.0, y: 0.0, z: 0.0 }, |d| Vec3 { x: d.x, y: d.y, z: d.z });
                        if *state == 4 {
                            *life = na.hit_life;
                            *state = 2;
                            it.msid = 2;
                            it.vel = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                        } else {
                            match m.rand.randi(5) {
                                0..=2 => {
                                    *life = na.hit_life;
                                    it.vel = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                                    *state = 2;
                                    it.msid = 2;
                                }
                                _ => {
                                    *life = na.hit_life;
                                    it.vel.y = NEEDLE_FALL[m.rand.randi(8) as usize].abs();
                                    setup_bounce(&mut m.rand, bx, bmin, bgrav);
                                    *state = 4;
                                    it.msid = 4;
                                }
                            }
                        }
                    }
                }
                _ => {
                    // UnkMotion2_Coll: stuck; the velocity is the line's own motion (zero for a
                    // static stage line).
                    it.vel = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                    if *line != -1 && m.ground.mp_lib_80054ed8(*line) {
                        let d = m.ground.mp_get_speed(*line, melee_world::vec::Vec3 { x: it.pos.x, y: it.pos.y, z: it.pos.z });
                        if let Some(d) = d {
                            it.vel = Vec3 { x: d.x, y: d.y, z: d.z };
                        }
                    }
                }
            }
            // Hit detection of an item is not ported: stop when it comes near a fighter.
            for (b, q) in m.players.iter().enumerate() {
                if b == owner || q.follower {
                    continue;
                }
                let c = q.fp.cur_pos;
                let (dx, dy) = (it.pos.x - c.x, it.pos.y - (c.y + 10.0));
                if dx * dx + dy * dy < 30.0 * 30.0 {
                    return Err(format!("a thrown needle comes near player {}: item hit detection not ported", b + 1));
                }
            }
            Ok(Some(false))
        }
        _ => Ok(None),
    }
}

/// The Coll callbacks' state changes once the floor under (air) / the floor end (ground) says so.
/// `ftCommon_AirToGroundStateChange` / `GroundToAirStateChange` keep the animation frame
/// (`anim_start = cur_anim_frame`); Cancel / End leave for Fall or Landing.
pub fn on_landed(g: &mut GroundColl, p: &mut Player, c: EntryCommon, rand: &mut HsdRand) -> Result<(), String> {
    let m = p.fp.motion_id;
    match m {
        SK_MS_SPECIAL_AIR_N_START | SK_MS_SPECIAL_AIR_N_LOOP => {
            ftcommon_8007d7fc(g, p, c)?;
            let msid = m - 4;
            let row = load_row(p, SK_SM_SPECIAL_N_START + (msid - SK_MS_SPECIAL_N_START) as u32)?;
            let frame = p.fp.cur_anim_frame;
            change_motion_state_at(p, msid, row.as_ref(), frame, rand)
        }
        SK_MS_SPECIAL_AIR_N_CANCEL => landing_enter_basic(g, p, c, rand),
        _ => {
            // SpecialAirNEnd_Coll.
            p.sk.count = 0;
            p.sk.n4 = false;
            landing_enter_basic(g, p, c, rand)
        }
    }
}

pub fn on_walked_off(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let m = p.fp.motion_id;
    match m {
        SK_MS_SPECIAL_N_START | SK_MS_SPECIAL_N_LOOP => {
            melee_fighter::ftcommon2::ftcommon_8007d5d4(&mut p.fp);
            let msid = m + 4;
            let row = load_row(p, SK_SM_SPECIAL_N_START + (msid - SK_MS_SPECIAL_N_START) as u32)?;
            let frame = p.fp.cur_anim_frame;
            change_motion_state_at(p, msid, row.as_ref(), frame, rand)
        }
        SK_MS_SPECIAL_N_CANCEL => end_to_fall(p, rand),
        _ => {
            // SpecialNEnd_Coll.
            p.sk.count = 0;
            p.sk.n4 = false;
            end_to_fall(p, rand)
        }
    }
}

/// `it_803F7020` / `it_803F6FA0` (the minimum fall speeds / bounce heights), `it_803F7040`
/// (gravity), `it_803F7060` (spin) and `it_803F7000` (drift).
const NEEDLE_FALL: [f32; 8] = [-2.0, -2.1, -2.2, -2.3, -2.4, -2.5, -2.6, -2.7];
const NEEDLE_GRAV: [f32; 8] = [-0.1, -0.12, -0.14, -0.18, -0.2, -0.22, -0.24, -0.26];
const NEEDLE_DRIFT: [f32; 8] = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4];

/// `itSeakNeedleThrown_SetupBounce` (the spin `xDD4` is render only but still draws).
fn setup_bounce(rand: &mut HsdRand, bx: &mut f32, bmin: &mut f32, bgrav: &mut f32) {
    let _spin_sign = rand.randi(2);
    let _spin = rand.randi(8);
    let sign = if rand.randi(2) == 0 { 1.0f32 } else { -1.0f32 };
    let v = NEEDLE_DRIFT[rand.randi(8) as usize];
    *bx = v * sign;
    *bmin = NEEDLE_FALL[rand.randi(8) as usize];
    *bgrav = NEEDLE_GRAV[rand.randi(8) as usize];
}
