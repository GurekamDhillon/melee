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
}

/// Work a fighter hands to the item procs.
#[derive(Clone, Copy, Debug)]
pub enum ItemCmd {
    /// `it_802AE8A8`: the blaster gun, at the fighter's position.
    SpawnGun { owner: usize, kind: i32, pos: Vec3 },
    /// `it_8029C6A4(angle, vel, owner, &muzzle, kind)`: a laser shot.
    Laser { owner: usize, kind: i32, angle: f32, speed: f32, pos: Vec3, parent: Vec3 },
}

#[derive(Clone, Copy, Debug)]
pub enum ItemClass {
    /// `flash`: `it_802AE1D0` armed the muzzle-flash accessory (`item->xDD4.foxblaster.gfx_spawn_var`).
    FoxGun { owner: usize, flash: bool },
    FoxLaser { angle: f32, speed: f32, life: f32 },
    /// A stage-owned item the sim does not step (Green Greens' blocks, kind 160): loaded from a
    /// snapshot, held unchanged, hashed.
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
        return Err(format!("player {}: ftFx_SpecialN_Enter on the ground (ftCommon_8007D7FC) not wired", p.slot + 1));
    }
    let row = load_row(p, smid)?;
    change_motion_state(p, msid, row.as_ref(), rand)?;
    // ftFox_SpecialN_InitializeState: Fighter_ClearCmdVars, ftAnim_8006EBA4.
    p.fp.cmd_vars = [0; 4];
    anim_step(p, rand)?;
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
fn fire_shot(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
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
    p.item_cmds.push(ItemCmd::Laser { owner: p.slot as usize, kind: da.x1C_FOX_BLASTER_SHOT_ITKIND as i32, angle: launch as f32, speed: da.x14_FOX_BLASTER_VEL, pos, parent });
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
fn spawn_step(last: Vec3, cur: Vec3) -> Vec3 {
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
                    fire_shot(p, rand)?;
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
        fire_shot(p, rand)?;
    }
    Ok(())
}

impl VsMatch {
    /// The item procs (after the fighters' procs of the frame): apply the fighters' item work,
    /// then run every item.
    pub(super) fn items_update(&mut self) -> Result<(), String> {
        for i in 0..self.players.len() {
            for c in std::mem::take(&mut self.players[i].item_cmds) {
                match c {
                    ItemCmd::SpawnGun { owner, kind, pos } => {
                        self.items.push(SimItem { kind, msid: 0, pos, vel: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, class: ItemClass::FoxGun { owner, flash: false } });
                    }
                    ItemCmd::Laser { owner: lo, kind, angle, speed, pos, parent } => {
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
                        self.items.push(SimItem { kind, msid: 0, pos, vel: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, class: ItemClass::FoxLaser { angle: a, speed, life: FOX_LASER_LIFETIME } });
                    }
                }
            }
        }
        let mut flashes: Vec<([f32; 3], f32, usize)> = Vec::new();
        let mut k = 0;
        while k < self.items.len() {
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
                ItemClass::FoxLaser { angle, speed, life } => {
                    // itFoxlaser_UnkMotion1_Anim -> Item_UpdateRayAnimation: velocity from the angle.
                    it.vel.x = *speed * melee_runtime::gekko::cosf(*angle);
                    it.vel.y = *speed * melee_runtime::gekko::sinf(*angle);
                    it.vel.z = 0.0;
                    // it_80273130: the life timer; the item proc stops there when it runs out.
                    *life -= 1.0;
                    if *life <= 0.0 {
                        dead = true;
                    } else {
                        it.pos.x += it.vel.x;
                        it.pos.y += it.vel.y;
                    }
                }
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
