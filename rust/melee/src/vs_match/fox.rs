//! Fox's neutral special (the blaster, ftfoxspecialn.c) and the two items it makes (the blaster
//! gun, itfoxblaster.c, and its laser shots, itfoxlaser.c).
//!
//! Items here are the hashed fields only (kind, msid, pos, vel: docs/rust-port/neutral-hash.md)
//! plus the little state the item procs need to decide their own lifetime. Not modelled: the
//! gun's model animation (`it_802AE200` & co.), its muzzle-flash effect (`it_802ADEF0` ->
//! `efSync_Spawn(1166)`, no RNG in the C trace), hit detection of a laser (stopped by
//! [`VsMatch::hit_guard`]'s sibling check in `items_update`) and its ground-line collision.

use super::*;

pub const FX_MS_SPECIAL_N_START: i32 = 341;
pub const FX_MS_SPECIAL_N_LOOP: i32 = 342;
pub const FX_MS_SPECIAL_N_END: i32 = 343;
pub const FX_MS_SPECIAL_AIR_N_START: i32 = 344;
pub const FX_MS_SPECIAL_AIR_N_LOOP: i32 = 345;
pub const FX_MS_SPECIAL_AIR_N_END: i32 = 346;
/// `ftFx_SM_SpecialNStart` = `ftCo_SM_Count` (295) .. `SpecialAirNEnd` (300).
const FX_SM_SPECIAL_N_START: u32 = 295;

pub fn is_fox_special_n(m: i32) -> bool {
    (FX_MS_SPECIAL_N_START..=FX_MS_SPECIAL_AIR_N_END).contains(&m)
}

fn is_fox_special_n_air(m: i32) -> bool {
    (FX_MS_SPECIAL_AIR_N_START..=FX_MS_SPECIAL_AIR_N_END).contains(&m)
}

/// `fp->u.fx.x222C_blasterGObj != NULL`, `fp->mv.fx.SpecialN.isBlasterLoop` and
/// `fp->accessory4_cb == ftFx_SpecialN_CreateBlasterShot`.
#[derive(Clone, Copy, Debug, Default)]
pub struct FoxVars {
    pub blaster: bool,
    pub is_loop: bool,
    pub accessory4: bool,
    /// The gun item's `foxblaster.set_sfx_var2` (it_802ADDD0's last visibility; 1 at spawn).
    pub gun_vis: i32,
    /// The gun item's `foxblaster.xDE0` (it_802AE538 has played the draw sound).
    pub gun_shot: bool,
    /// Reflector (foxlw.rs): `accessory4_cb` (0 none, 1 Start / 2 Loop / 3 Reflect GFX), `mv.fx.SpecialLw`, `reflect_hit_cb` set.
    pub lw_gfx: u8,
    pub lw_release_lag: i32,
    pub lw_release: bool,
    pub lw_gravity_delay: i32,
    pub lw_reflect: bool,
}

/// Work a fighter hands to the item procs.
#[derive(Clone, Copy, Debug)]
pub enum ItemCmd {
    /// `it_802AE8A8`: the blaster gun, at the fighter's position.
    SpawnGun { owner: usize, kind: i32, pos: Vec3 },
    /// `it_8029C6A4(angle, vel, owner, &muzzle, kind)`: a laser shot.
    Laser { owner: usize, kind: i32, angle: f32, speed: f32, pos: Vec3, parent: Vec3, slink1: bool },
    /// `it_802B19AC`: Sheik's held needle bundle, at the fighter's position.
    SeakHeld { owner: usize, pos: Vec3 },
}

#[derive(Clone, Copy, Debug)]
pub enum ItemClass {
    /// `flash`: `it_802AE1D0` armed the muzzle-flash accessory (`item->xDD4.foxblaster.gfx_spawn_var`).
    FoxGun { owner: usize, flash: bool },
    /// `ray`: `item->xDD4_itemVar.ray.scale` (the model's Z scale); `hits`: its hit capsules.
    FoxLaser { angle: f32, speed: f32, life: f32, owner: usize, ray: f32, hits: LaserHits },
    /// `It_Kind_Seak_NeedleHeld`: alive while the owner's `u.sk.x4` is set.
    SeakHeld { owner: usize },
    /// `It_Kind_Seak_NeedleThrow` in its flying state (msid 0): `fresh` = created this frame
    /// (its anim / phys / coll procs ran before the fighter's accessory proc made it).
    SeakThrown { owner: usize, life: f32, fresh: bool, state: u8, line: i32, bx: f32, bmin: f32, bgrav: f32 },
    /// Green Greens' breakable block (`grMaterial_801C8CFC` -> `it_802E6AEC`, `It_Kind_Yaku`).
    GreensBlock,
    /// Samus' missile (samus.rs): `fresh` = made this frame, its item procs start the next one.
    SsMissile { owner: usize, smash: bool, life: f32, x8: f32, v0: f32, v1: f32, v3: f32, born: u32, fresh: bool, facing: f32, jref: JObjRef, a: super::samus::MissileAttrs, hb: u8, x58: Vec3 },
    /// A stage-owned item the sim does not step (snapshot loading): held unchanged, hashed.
    Static,
}

#[derive(Clone, Copy, Debug)]
pub struct SimItem {
    pub kind: i32,
    pub msid: i32,
    pub pos: Vec3,
    pub vel: Vec3,
    pub class: ItemClass,
}


/// The hit capsules of a Fox / Falco laser (`item->x5D4_hitboxes[i].hit`): the fields the CPU's threat
/// check reads (`ftCo_800BB220`): `state` (`HitCapsule_Disabled` 0 .. `Unk3` 3), the capsule's two
/// end positions `x4C` (now) and `x58` (the previous update).
///
/// The hitboxes are created by the article's animation script (cmd `it_802790C0`, damage 0): the model's
/// root joint, `size` 300/256, and `b_offset.z` per box. The constants below were read off the C build
/// (a debug print of the item in `ftCo_800BB220`) rather than parsed from `PlFx.dat` / `PlFc.dat`:
/// Fox's shot has three boxes, Falco's four (the fourth of Fox's stays Disabled).
#[derive(Clone, Copy, Debug)]
pub struct LaserHits {
    pub n: usize,
    pub state: [u8; 4],
    pub x4c: [Vec3; 4],
    pub x58: [Vec3; 4],
    /// False when the hit state is not modelled (the shot was made by the s_link 9 accessory, whose
    /// item procs of s_link 0..5 did not run in the creating frame).
    pub valid: bool,
    /// `hit->victims`: the players this shot has hit (bit per player index).
    pub victims: u8,
    /// `hit->damage` after the owner's staling (`it_80272460` -> `ft_80089228`).
    pub dmg: f32,
    /// `item->xD88_attackID` / `xD8C_attack_instance`: the owner's at the shot.
    pub atk_id: i32,
    pub atk_inst: u16,
    /// `item->xC34_damageDealt != 0`: the shot hit a fighter and is destroyed at s_link 14
    /// (`OnGiveDamageThink`: `itFoxLaser_Logic94_Clanked` returns true).
    pub dealt: bool,
}

/// `hit->scale` (`size` 300 / 256).
pub const LASER_HIT_SCALE: f32 = f32::from_bits(0x3F95_FD8B);
/// The attribute table's `scale` (`FoxLaserAttr`, maximum Z scale), [3].
const LASER_MAX_SCALE: f32 = 3.0;

/// `hit->b_offset.z` of the laser's boxes.
pub fn laser_offsets(kind: i32) -> &'static [f32] {
    use std::sync::OnceLock;
    static FOX: OnceLock<[f32; 3]> = OnceLock::new();
    static FALCO: OnceLock<[f32; 4]> = OnceLock::new();
    if kind == melee_types::It_Kind_Fox_Laser as i32 {
        FOX.get_or_init(|| [f32::from_bits(0xBF47_FCB9), f32::from_bits(0xC069_3C2D), f32::from_bits(0xC0D0_3C96)])
    } else {
        FALCO.get_or_init(|| [f32::from_bits(0xBF47_FCB9), f32::from_bits(0xC069_3C2D), f32::from_bits(0xC0D0_3C96), f32::from_bits(0xC115_FD8B)])
    }
}

/// `lb_8000B1CC(jobj, &(0, 0, off), &out)` for the laser's root joint: the translate is the item position,
/// the rotation Rz * Ry(beta) * Rx(alpha), the Z scale `scale`.
fn laser_joint(pos: Vec3, off: f32, scale: f32, alpha: f32, beta: f32) -> Vec3 {
    use melee_runtime::gekko::{cosf, sinf};
    let pz = off * scale;
    let (sa, ca, sb, cb) = (sinf(alpha), cosf(alpha), sinf(beta), cosf(beta));
    // Rx: (0, -sa * pz, ca * pz); then Ry: x = sb * z, z = cb * z.
    let y1 = -sa * pz;
    let z1 = ca * pz;
    Vec3 { x: pos.x + sb * z1, y: pos.y + y1, z: pos.z + cb * z1 }
}

/// The laser's facing for a launch angle (`it_8029C504`: `right_facing`).
fn laser_facing(angle: f32) -> f32 {
    if !((angle as f64) < core::f64::consts::FRAC_PI_2) && !((angle as f64) > core::f64::consts::PI * 3.0 / 2.0) {
        -1.0
    } else {
        1.0
    }
}

/// The hit capsules of a laser made this frame by a fighter proc of s_link 1: Fox's article script
/// also steps the boxes once (`it_80272560` -> `it_8027129C` x4: Enabled -> Unk2 with the joint at the
/// position and Z scale 1 of the creation); Falco's leaves them Enabled.
pub(super) fn laser_hits_new(kind: i32, pos: Vec3, angle: f32, slink1: bool) -> LaserHits {
    let offs = laser_offsets(kind);
    let z = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut h = LaserHits { n: offs.len(), state: [0; 4], x4c: [z; 4], x58: [z; 4], valid: slink1, victims: 0, dmg: 3.0, atk_id: 18, atk_inst: 0, dealt: false };
    for i in 0..offs.len() {
        h.state[i] = 1;
        if kind == melee_types::It_Kind_Fox_Laser as i32 {
            // The joint at creation: HSD_JObjSetFacingDirItem sets Y only (alpha 0).
            let beta = (core::f64::consts::FRAC_PI_2 * laser_facing(angle) as f64) as f32;
            let j = laser_joint(pos, offs[i], 1.0, 0.0, beta);
            h.x4c[i] = j;
            h.x58[i] = j;
            h.state[i] = 2;
        }
    }
    h
}

/// `it_8027137C` (the item's s_link 11 hit update) for a laser, with the joint as the item procs left it.
fn laser_hits_step(h: &mut LaserHits, kind: i32, pos: Vec3, vel: Vec3, scale: f32) {
    if !h.valid {
        return;
    }
    let offs = laser_offsets(kind);
    // Item_UpdateRayAnimation's rotations.
    let facing: f32 = if vel.x > 0.0 { 1.0 } else { -1.0 };
    let beta = (core::f64::consts::FRAC_PI_2 * facing as f64) as f32;
    let vx = if facing == 1.0 { -vel.x } else { vel.x };
    let alpha = (core::f64::consts::PI + melee_runtime::lbtrigf::atan2f(vel.y, vx) as f64) as f32;
    for i in 0..h.n {
        let j = laser_joint(pos, offs[i], scale, alpha, beta);
        match h.state[i] {
            1 => {
                h.x4c[i] = j;
                h.x58[i] = j;
                h.state[i] = 2;
            }
            2 | 3 => {
                h.state[i] = 3;
                h.x58[i] = h.x4c[i];
                h.x4c[i] = j;
            }
            _ => {}
        }
    }
}

/// The lasers fighters made in this frame's s_link 1 procs (their items exist for the CPU controllers of
/// s_link 2, whose item procs of s_link 1 have run: the pending commands are applied after the frame).
pub(super) fn pending_lasers(players: &[Player]) -> Vec<SimItem> {
    let mut v = Vec::new();
    for pl in players {
        for c in &pl.item_cmds {
            if let ItemCmd::Laser { owner, kind, angle, speed, pos, parent, slink1 } = *c {
                if !slink1 {
                    continue;
                }
                let mut a = angle;
                while a < 0.0 {
                    a += core::f32::consts::TAU;
                }
                while a > core::f32::consts::TAU {
                    a -= core::f32::consts::TAU;
                }
                let pos = spawn_step(parent, pos);
                v.push(SimItem { kind, msid: 0, pos, vel: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, class: ItemClass::FoxLaser { angle: a, speed, life: FOX_LASER_LIFETIME, owner, ray: 0.0, hits: laser_hits_new(kind, pos, a, true) } });
            }
        }
    }
    v
}

/// The laser's lifetime: `FoxLaserAttr.lifetime` (the article's special attributes).
const FOX_LASER_LIFETIME: f32 = 35.0;

/// `ftFx_SpecialN_GetBlasterAction` for the SpecialN states (9 otherwise).
fn blaster_action(fp: &Fighter) -> i32 {
    if is_fox_special_n(fp.motion_id) {
        fp.motion_id - FX_MS_SPECIAL_N_START
    } else {
        9
    }
}

/// `ftFx_SpecialN_Enter` / `ftFx_SpecialAirN_Enter`.
pub fn special_n_enter(p: &mut Player, air: bool, rand: &mut HsdRand) -> Result<(), String> {
    let da = p.fx_attrs.ok_or("no Fox attrs")?;
    let (msid, smid) = if air { (FX_MS_SPECIAL_AIR_N_START, FX_SM_SPECIAL_N_START + 3) } else { (FX_MS_SPECIAL_N_START, FX_SM_SPECIAL_N_START) };
    if !air {
        // ftCommon_8007D7FC(fp).
        let (g, c) = (unsafe { &mut *p.gc }, p.common_c);
        ftcommon_8007d7fc(g, p, c)?;
    }
    let row = load_row(p, smid)?;
    change_motion_state(p, msid, row.as_ref(), rand)?;
    // ftFox_SpecialN_InitializeState: Fighter_ClearCmdVars, ftAnim_8006EBA4.
    p.fp.cmd_vars = [0; 4];
    anim_step(p, rand)?;
    if !air {
        p.fp.gr_vel = 0.0;
        p.fp.self_vel = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
    }
    p.fx.is_loop = false;
    // ftFox_SpecialN_SpawnBlaster: it_802AE8A8(facing_dir, gobj, &cur_pos, RThumbNb, gun kind).
    let pos = p.fp.cur_pos;
    p.item_cmds.push(ItemCmd::SpawnGun { owner: p.slot as usize, kind: da.x20_FOX_BLASTER_GUN_ITKIND as i32, pos: Vec3 { x: pos.x, y: pos.y, z: 0.0 } });
    p.fx.blaster = true;
    // it_802AE7B8: set_sfx_var2 = 1, xDE0 = false.
    p.fx.gun_vis = 1;
    p.fx.gun_shot = false;
    Ok(())
}

/// The shot: `ftFox_SpecialN_PrepareBlasterShot` + `FireBlasterShot`.
fn fire_shot(p: &mut Player, rand: &mut HsdRand, slink1: bool) -> Result<(), String> {
    let da = p.fx_attrs.ok_or("no Fox attrs")?;
    // ftFx_SpecialN_FtGetHoldJoint: lb_8000B1CC(RThumbNb part, {0, 1.2325, 4.2636}, &pos).
    let part = bone_index(p, FT_PART_RTHUMBNB);
    let j = p.skel.part(part);
    let mut pos = p.skel.world_point(j, Vec3 { x: 0.0, y: 1.2325000762939453, z: 4.263599872589111 });
    pos.z = 0.0;
    let ang = da.x10_FOX_BLASTER_ANGLE;
    let launch: f64 = if p.fp.facing_dir == 1.0 { ang as f64 } else { core::f64::consts::PI - ang as f64 };
    // Item_InitSpawnPositionFromParent: ftLib_80086990 = the fighter's position + (0, mid ECB height, 0).
    let fp = &p.fp;
    let mid = 0.5f32 * (fp.coll_data.ecb.top.y + fp.coll_data.ecb.bottom.y);
    let parent = Vec3 { x: fp.cur_pos.x + 0.0, y: fp.cur_pos.y + mid, z: fp.cur_pos.z + 0.0 };
    p.item_cmds.push(ItemCmd::Laser { owner: p.slot as usize, kind: da.x1C_FOX_BLASTER_SHOT_ITKIND as i32, angle: launch as f32, speed: da.x14_FOX_BLASTER_VEL, pos, parent, slink1 });
    // ftFox_SpecialN_FireBlasterShot: the shot's sound (foxSFX / falcoSFX by facing).
    let left = (p.fp.facing_dir == -1.0) as usize;
    match p.fp.kind {
        melee_types::Ft_Kind_Fox => super::sound::play_sfx(p, rand, [110103, 110106][left], 0x7F, 0x40),
        melee_types::Ft_Kind_Falco => super::sound::play_sfx(p, rand, [100099, 100102][left], 0x7F, 0x40),
        _ => {}
    }
    Ok(())
}

/// `mpColl_80043754`'s position stepping from `last` to `cur` (no stage contact).
pub(super) fn spawn_step(last: Vec3, cur: Vec3) -> Vec3 {
    let mut vel = Vec3 { x: cur.x - last.x, y: cur.y - last.y, z: cur.z - last.z };
    let x = if vel.x.abs() > vel.y.abs() { vel.x.abs() } else { vel.y.abs() };
    let steps = if x > 6.0 {
        let s = melee_runtime::cabi::f2i32((x / 6.0) as f64) + 1;
        vel.x /= s as f32;
        vel.y /= s as f32;
        vel.z /= s as f32;
        s
    } else {
        1
    };
    let mut p = last;
    for _ in 0..steps {
        p.x += vel.x;
        p.y += vel.y;
        p.z += vel.z;
    }
    p
}

/// `FtPart_RThumbNb`.
const FT_PART_RTHUMBNB: u32 = 49;

/// `it_802ADDD0(gun, visibility)`'s sound: the put-away sound when the visibility becomes 2.
fn gun_visibility(p: &mut Player, vis: i32) {
    if !p.fx.blaster || p.fx.gun_vis == vis {
        return;
    }
    p.fx.gun_vis = vis;
    if vis == 2 {
        gun_sfx(p, 0x1AE14, 0x18700);
    }
}

/// `itFoxBlaster_PlaySFX` -> `Item_8026AE84(item, sfx, 0x7F, 0x40)`: the gun item's own sound
/// (fox or falco id by the item kind; the item has no positional sound object).
fn gun_sfx(p: &Player, fox: i32, falco: i32) {
    let id = if p.fp.kind == melee_types::Ft_Kind_Falco { falco } else { fox };
    melee_audio::sim::lbaudioax_800237a8(melee_audio::sim::lbaudioax_800233ec(id), 0x7F, 0x40);
}

/// `ftFox_SpecialN_UpdateBlaster`: it_802ADDD0(gun, 1), then cmd_vars[3] == 1 -> it_802AE538
/// (the first time only: xDE0).
fn update_blaster(p: &mut Player) {
    gun_visibility(p, 1);
    if p.fp.cmd_vars[3] == 1 && p.fx.blaster {
        p.fp.cmd_vars[3] = 0;
        if !p.fx.gun_shot {
            gun_sfx(p, 0x1AE05, 0x186F1);
            p.fx.gun_shot = true;
        }
    }
}

/// The Anim callbacks of the six states.
pub fn anim(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let m = p.fp.motion_id;
    let air = is_fox_special_n_air(m);
    let state = m - if air { FX_MS_SPECIAL_AIR_N_START } else { FX_MS_SPECIAL_N_START };
    let da = p.fx_attrs.ok_or("no Fox attrs")?;
    match state {
        0 | 1 => {
            // Start (0) / Loop (1): UpdateBlaster, then the transitions.
            update_blaster(p);
            if !p.skel.frames_remaining() {
                let base = if air { FX_MS_SPECIAL_AIR_N_START } else { FX_MS_SPECIAL_N_START };
                if state == 0 {
                    // StartAnimation: the loop state (SkipModel | KeepGfx), accessory4_cb armed.
                    let row = load_row(p, FX_SM_SPECIAL_N_START + (base - FX_MS_SPECIAL_N_START + 1) as u32)?;
                    p.keep_gfx = true;
                    change_motion_state(p, base + 1, row.as_ref(), rand)?;
                    p.fx.accessory4 = true;
                    gun_visibility(p, 1);
                } else if p.fx.is_loop {
                    // BeginLoopTransition (SkipAttackCount | SkipModel | KeepGfx) + FinishLoopTransition.
                    let row = load_row(p, FX_SM_SPECIAL_N_START + (base - FX_MS_SPECIAL_N_START + 1) as u32)?;
                    p.keep_gfx = true;
                    change_motion_state(p, base + 1, row.as_ref(), rand)?;
                    p.fx.accessory4 = true;
                    p.fx.is_loop = false;
                    gun_visibility(p, 1);
                } else {
                    // End: SkipModel | KeepGfx, then cmd_vars[1] = 1.
                    let row = load_row(p, FX_SM_SPECIAL_N_START + (base - FX_MS_SPECIAL_N_START + 2) as u32)?;
                    p.keep_gfx = true;
                    change_motion_state(p, base + 2, row.as_ref(), rand)?;
                    p.fp.cmd_vars[1] = 1;
                    gun_visibility(p, 1);
                }
            }
            if state == 1 {
                // The Loop callback also fires a shot flagged in cmd_vars[2] right here.
                if p.fp.cmd_vars[2] != 0 {
                    p.fp.cmd_vars[2] = 0;
                    fire_shot(p, rand, true)?;
                }
            }
            let _ = da;
            Ok(())
        }
        2 => {
            // End: UpdateEndAnimation, then the item is released and Fall / landing lag.
            // cmd_vars[1] == 2: ftpickupitem_80094818(gobj, 0), a no-op without a held item
            // (fp->item_gobj and x1978 are NULL: the gun is not a held item of the fighter).

            gun_visibility(p, p.fp.cmd_vars[1] as i32);
            if p.fp.cmd_vars[3] == 2 && p.fx.blaster {
                p.fp.cmd_vars[3] = 0;
            }
            if !p.skel.frames_remaining() {
                p.fx.blaster = false;
                if air {
                    if da.x18_FOX_BLASTER_LANDING_LAG == 0.0 {
                        fall_enter(p, rand)?;
                    } else {
                        return Err("ftCo_80096900 (blaster landing lag) not ported".into());
                    }
                } else {
                    ft_8008a2bc(p, rand)?;
                }
            }
            Ok(())
        }
        _ => Err(format!("fox special N state {state} not ported")),
    }
}

/// The three IASAs: a second B press inside the window loops the blaster.
pub fn iasa(p: &mut Player) {
    // Start / Loop only (End has no IASA).
    if !matches!(p.fp.motion_id, FX_MS_SPECIAL_N_END | FX_MS_SPECIAL_AIR_N_END) && p.fp.cmd_vars[0] != 0 && p.fp.input.pressed_buttons & input::HSD_PAD_B != 0 {
        p.fx.is_loop = true;
    }
}

/// `ftFx_SpecialN_CreateBlasterShot` (accessory4_cb, s_link 9).
pub fn accessory4(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    if p.fx.accessory4 && p.fp.cmd_vars[2] != 0 {
        p.fp.cmd_vars[2] = 0;
        fire_shot(p, rand, false)?;
    }
    Ok(())
}

impl VsMatch {
    /// The item procs (after the fighters' procs of the frame): apply the fighters' item work,
    /// then run every item.
    pub(super) fn items_update(&mut self) -> Result<(), String> {
        for i in 0..self.players.len() {
            let mut rest = Vec::new();
            for c in std::mem::take(&mut self.players[i].item_cmds) {
                match c {
                    ItemCmd::SpawnGun { owner, kind, pos } => {
                        self.items.push(SimItem { kind, msid: 0, pos, vel: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, class: ItemClass::FoxGun { owner, flash: false } });
                    }
                    ItemCmd::SeakHeld { owner, pos } => {
                        self.items.push(SimItem { kind: melee_types::It_Kind_Seak_NeedleHeld as i32, msid: 0, pos, vel: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, class: ItemClass::SeakHeld { owner } });
                    }
                    // The lasers are made by `lasers_step` (s_link 13 of the creating frame).
                    c @ ItemCmd::Laser { .. } => rest.push(c),
                }
            }
            self.players[i].item_cmds = rest;
        }
        let mut flashes: Vec<([f32; 3], f32, usize)> = Vec::new();
        let mut k = 0;
        while k < self.items.len() {
            if matches!(self.items[k].class, ItemClass::SsMissile { .. }) {
                if self.missile_step(k)? {
                    self.items.remove(k);
                } else {
                    k += 1;
                }
                continue;
            }
            let mut it = self.items[k];
            let mut dead = false;
            match &mut it.class {
                ItemClass::FoxGun { owner, flash } => {
                    let p = &mut self.players[*owner];
                    // itFoxblaster_UnkMotion8_Anim: follow the fighter's SpecialN phase.
                    if !matches!(it.msid, 2 | 5..=8) {
                        let state = blaster_action(&p.fp);
                        if it.msid != state {
                            it.msid = state;
                        }
                    }
                    let action = blaster_action(&p.fp);
                    if action == 9 || !p.fx.blaster {
                        // clear_blaster: the fighter forgets it, the item is destroyed.
                        p.fx.blaster = false;
                        dead = true;
                    } else if *flash {
                        // it_802ADF10 (the item's accessory): the muzzle flash, efSync_Spawn(1166 = 0x48E)
                        // -> efLib_Create_Attach_Pos(0xBBD) at the fighter's hold joint, rot Y = pi/2,
                        // rot Z = the angle from the item's hold point to the fighter's.
                        *flash = false;
                        let owner = *owner;
                        let part = bone_index(&self.players[owner], FT_PART_RTHUMBNB);
                        let pl = &mut self.players[owner];
                        let j = pl.skel.part(part);
                        let ft_hold = pl.skel.world_point(j, Vec3 { x: 0.0, y: 1.2325000762939453, z: 4.263599872589111 });
                        let it_hold = pl.skel.world_point(j, Vec3 { x: 0.0, y: 1.2325000762939453, z: 0.013600001111626625 });
                        let angle = melee_runtime::lbtrigf::atan2f(ft_hold.y - it_hold.y, ft_hold.x - it_hold.x);
                        flashes.push(([ft_hold.x, ft_hold.y, ft_hold.z], angle, owner));
                    }
                }
                ItemClass::Static => {}
                ItemClass::SeakHeld { .. } | ItemClass::SeakThrown { .. } => {} // sheik::item_proc, below
                ItemClass::GreensBlock => {}
                ItemClass::SsMissile { .. } => {}
                ItemClass::FoxLaser { .. } => {} // lasers_step, ahead of the fighters' s_link 13
            }
            if let Some(d) = super::sheik::item_proc(self, &mut it)? {
                dead = d;
            }
            if dead {
                self.items.remove(k);
            } else {
                self.items[k] = it;
                k += 1;
            }
        }
        for (pos, angle, owner) in flashes {
            self.ef_create_rot(0xBBD, owner, 1.0, Some(pos), Some(core::f32::consts::FRAC_PI_2), Some(angle))?;
        }
        Ok(())
    }


    /// The lasers' item procs of a frame, run ahead of the fighters' s_link 13 (`Fighter_8006CB94`: the
    /// hit detection reads the shots as their s_link 1 (anim), 4 (phys) and 11 (hit capsules) procs of
    /// this frame left them): the shots a fighter made this frame (`it_8029C504`), then every shot's step.
    pub(super) fn lasers_step(&mut self) -> Result<(), String> {
        for i in 0..self.players.len() {
            let mut rest = Vec::new();
            for c in std::mem::take(&mut self.players[i].item_cmds) {
                match c {
                    ItemCmd::Laser { owner: lo, kind, angle, speed, pos, parent, slink1 } => {
                        // it_802AE1D0(fp->u.fx.x222C_blasterGObj): arm the gun's muzzle flash.
                        for g in &mut self.items {
                            if let ItemClass::FoxGun { owner, flash } = &mut g.class {
                                if *owner == lo {
                                    *flash = true;
                                }
                            }
                        }
                        // it_8029C504: Item_NormalizeAngle(&angle), the spawn at the muzzle.
                        let mut a = angle;
                        while a < 0.0 {
                            a += core::f32::consts::TAU;
                        }
                        while a > core::f32::consts::TAU {
                            a -= core::f32::consts::TAU;
                        }
                        // it_80276100: the item's collision step from the parent to the muzzle
                        // (mpColl_800471F8 over more than 6 units substeps; each substep adds
                        // vel / steps, which rounds). The item's ECB and the stage are not consulted.
                        let pos = spawn_step(parent, pos);
                        let mut hits = laser_hits_new(kind, pos, a, slink1);
                        // Item_InitSpawnCommonFields: the shot carries the owner's attack id and instance; the
                        // article script's create_hitbox (it_80272460) stales the 3% by the owner's queue.
                        let of = &self.players[lo].fp;
                        hits.atk_id = of.x2068_attackID;
                        hits.atk_inst = of.x206C_attack_instance;
                        hits.dmg = melee_fighter::stale::apply_staling(of, of.x2068_attackID, of.x206C_attack_instance as i32, 3.0);
                        self.items.push(SimItem { kind, msid: 0, pos, vel: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, class: ItemClass::FoxLaser { angle: a, speed, life: FOX_LASER_LIFETIME, owner: lo, ray: 0.0, hits } });
                    }
                    c => rest.push(c),
                }
            }
            self.players[i].item_cmds = rest;
        }
        let mut k = 0;
        while k < self.items.len() {
            let mut it = self.items[k];
            let mut dead = false;
            if let ItemClass::FoxLaser { angle, speed, life, ray, hits, .. } = &mut it.class {
                // itFoxlaser_UnkMotion1_Anim -> Item_UpdateRayAnimation: velocity from the angle.
                it.vel.x = *speed * melee_runtime::gekko::cosf(*angle);
                it.vel.y = *speed * melee_runtime::gekko::sinf(*angle);
                it.vel.z = 0.0;
                // The growing model scale (divisor 11.25), clamped to the attribute's maximum.
                *ray += speed.abs() / 11.25f32;
                if *ray > LASER_MAX_SCALE {
                    *ray = LASER_MAX_SCALE;
                }
                if *ray < 1e-5 {
                    *ray = 1e-3;
                }
                // it_80273130: the life timer; the item proc stops there when it runs out.
                *life -= 1.0;
                if *life <= 0.0 {
                    dead = true;
                } else {
                    it.pos.x += it.vel.x;
                    it.pos.y += it.vel.y;
                    // s_link 11, it_8027137C: the hit capsules follow the joint.
                    laser_hits_step(hits, it.kind, it.pos, it.vel, *ray);
                }
            }
            if dead {
                self.items.remove(k);
            } else {
                self.items[k] = it;
                k += 1;
            }
        }
        Ok(())
    }

    /// `item_count` and `item_sum` of the neutral hash.
    pub(super) fn item_words(&self) -> (u32, u32) {
        let mut sum = 0u32;
        for it in &self.items {
            let mut ip = melee_types::Item::zeroed();
            ip.kind = it.kind as _;
            ip.msid = it.msid as _;
            ip.pos = melee_types::Vec { x: it.pos.x, y: it.pos.y, z: it.pos.z };
            ip.x40_vel = melee_types::Vec { x: it.vel.x, y: it.vel.y, z: it.vel.z };
            sum = sum.wrapping_add(melee_types::neutral_hash::nh_item(&ip));
        }
        (self.items.len() as u32, sum)
    }
}
