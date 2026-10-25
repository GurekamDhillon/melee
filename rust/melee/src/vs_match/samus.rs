//! Samus' side special, the missile (`ftsamusspecials.c`, `ftsamusspecialn.c` `ftSs_SpecialS_8012A074`, and the
//! missile article `itsamusmissile.c`). The ground states only: the airborne ones return `Err`.

use super::*;
use super::fox::{ItemClass, SimItem};

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
    /// `fp->u.ss.x2238`: the missiles thrown so far.
    pub x2238: u32,
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


// ---- the missile article (itsamusmissile.c) ------------------------------------------------------

/// `itSamusMissileAttributes` (`ft_data->x48_items[2]`'s special attributes), x0..x3C.
#[derive(Clone, Copy, Debug)]
pub struct MissileAttrs(pub [f32; 16]);

impl MissileAttrs {
    pub fn from_dat(d: &[u8], ft_data: u32) -> MissileAttrs {
        let art = be32(d, be32(d, ft_data + 0x48) + 8);
        let special = be32(d, art + 4);
        let mut a = [0.0f32; 16];
        for (i, v) in a.iter_mut().enumerate() {
            *v = f32::from_bits(be32(d, special + 4 * i as u32));
        }
        MissileAttrs(a)
    }
}

/// The ItCo `xF8` lifetime of an exploding item (`it_8027518C`).
const EXPLOSION_LIFE: f32 = 80.0;

/// The missile's hit capsule: `hit->scale` (read off the C build's item).
pub const MISSILE_HIT_SCALE: f32 = f32::from_bits(0x3FF9_FBE7);

/// `It_Kind_Samus_Missile`.
pub const IT_KIND_SAMUS_MISSILE: i32 = 95;

/// The fighter-owned part of `ftSs_SpecialS_8012A074` and `ftSs_SpecialS_8012A168`.
pub fn missile_cmd(p: &mut Player) -> Option<(Vec3, bool)> {
    if !p.ss.accessory4 {
        return None;
    }
    let fp = &mut *p.fp;
    if unsafe { fp.anon_2210.anon_0.throw_flags_b0() } == 0 {
        return None;
    }
    // ftCheckThrowB0 clears the flag.
    unsafe { fp.anon_2210.anon_0.set_throw_flags_b0(0) };
    p.ss.x2238 += 1;
    let a = p.ss_attrs.expect("Samus attrs");
    let m = p.fp.motion_id;
    let smash = !(m == SS_MS_SPECIAL_S || m == SS_MS_SPECIAL_AIR_S);
    // lb_8000B1CC(fp->parts[FtPart_56].joint, NULL, &position); position.x += x34 * facing_dir.
    let j = p.skel.part(56);
    let mut pos = p.skel.world_pos(j);
    pos.x += a.x34 * p.fp.facing_dir;
    p.ss.accessory4 = false;
    Some((pos, smash))
}

impl VsMatch {
    /// `ftSs_SpecialS_8012A074` for player `i` (accessory4_cb, s_link 9).
    pub(super) fn samus_accessory4(&mut self, i: usize) -> Result<(), String> {
        let Some((pos, smash)) = missile_cmd(&mut self.players[i]) else { return Ok(()) };
        if smash {
            return Err("it_802B6A60: the smash missile is not ported".into());
        }
        let p = &self.players[i];
        let attrs = MissileAttrs::from_dat(&p.files.dat.data, p.files.ft_data);
        let facing = p.fp.facing_dir;
        let owner_count = p.ss.x2238;
        // Item_InitSpawnPositionFromParent (prev_pos = the position with z = 0, pos = the fighter's body position
        // ftLib_80086990) and the initial collision step between the two (as the Fox laser's).
        let pos = {
            let fp = &p.fp;
            let mid = 0.5f32 * (fp.coll_data.ecb.top.y + fp.coll_data.ecb.bottom.y);
            let parent = Vec3 { x: fp.cur_pos.x + 0.0, y: fp.cur_pos.y + mid, z: fp.cur_pos.z + 0.0 };
            super::fox::spawn_step(parent, Vec3 { x: pos.x, y: pos.y, z: 0.0 })
        };
        // it_802B66A8: vel.x = xC * facing, the lifetime x4, state 0, the smoke generator on the item's joint.
        let a = attrs.0;
        if std::env::var_os("MELEE_GOLDEN_ITM").is_some() { eprintln!("MISSILE pos {:08X} {:08X} {:08X} cur {:08X} {:08X} facing {} x34 {}", pos.x.to_bits(), pos.y.to_bits(), pos.z.to_bits(), p.fp.cur_pos.x.to_bits(), p.fp.cur_pos.y.to_bits(), facing, p.ss_attrs.unwrap().x34); }
        let jref: JObjRef = (0xC0 << 24) | (self.items.len() as u32 + 1);
        self.joint_mtx.insert(jref, translation_mtx(pos));
        let mut host = Host { rand: &mut self.rand, joint_mtx: &self.joint_mtx, appsrt_hook_lists: &self.appsrt_hook_lists };
        // efSync_Spawn(0x485) -> hsd_8039EFAC(0, 2, 0x7DE, item joint).
        self.ps.gen_create_on_jobj(&mut host, 0, 2, 0x7DE, Some(jref));
        self.items.push(SimItem {
            kind: IT_KIND_SAMUS_MISSILE,
            msid: 0,
            pos,
            vel: Vec3 { x: a[3] * facing, y: 0.0, z: 0.0 },
            class: ItemClass::SsMissile { owner: i, smash: false, life: a[1], x8: 0.0, v0: 0.0, v1: 0.0, v3: 0.0, born: owner_count, fresh: true, facing, jref, a: attrs, hb: 2, x58: pos },
        });
        // ftSs_SpecialS_8012A168: efSync_Spawn(1155) -> efLib_CreateGenerator(0x7D7, &position) (bank 2).
        let mut host = Host { rand: &mut self.rand, joint_mtx: &self.joint_mtx, appsrt_hook_lists: &self.appsrt_hook_lists };
        if let Some(g) = self.ps.gen_create(&mut host, 0, 2, 0x7D7) {
            self.ps.gen_mut(g).pos = pos;
        }
        let fp = &mut self.players[i].fp;
        if fp.x2219_b0() == 0 {
            fp.set_x2219_b0(1);
        }
        Ok(())
    }

    /// The missile's item procs for one frame (`itSamusmissile_UnkMotion0/3_Anim`, `_Phys`, the item physics).
    pub(super) fn missile_step(&mut self, k: usize) -> Result<bool, String> {
        let mut it = self.items[k];
        let ItemClass::SsMissile { owner, smash, mut life, mut x8, mut v0, mut v1, v3, born, fresh, facing, jref, a, hb, x58 } = it.class else { return Ok(false) };
        if fresh {
            if let ItemClass::SsMissile { fresh, .. } = &mut self.items[k].class {
                *fresh = false;
            }
            return Ok(false);
        }
        let at = a.0;
        if smash {
            return Err("the smash missile is not ported".into());
        }
        if it.msid == 2 {
            // itSamusmissile_UnkMotion3_Anim = it_802751D8: the explosion's lifetime.
            life -= 1.0;
            if let ItemClass::SsMissile { life: l, .. } = &mut self.items[k].class {
                *l = life;
            }
            return Ok(life <= 0.0);
        }
        // it_802B63F8.
        v0 = if life > at[1] - at[2] { 1.0 } else { 0.0 };
        let mut explode = false;
        if life == at[1] - at[2] {
            // efLib_DestroyAll(gobj): the smoke generator.
            self.missile_release_smoke(jref);
        }
        if life <= at[9] - at[10] {
            v1 = 1.0;
        }
        if life <= 0.0 {
            explode = true;
        } else {
            life -= 1.0;
        }
        // isSamusmissile_MotionAnim: the owner threw another missile meanwhile.
        if !explode && v3 == 0.0 {
            let c = self.players[owner].ss.x2238;
            if c.wrapping_sub(born) >= 2 {
                explode = true;
            }
        }
        if explode {
            return self.missile_explode(k, life, x8, v0, v1);
        }
        // itSamusmissile_UnkMotion0_Phys.
        let mut vel = it.vel;
        if v0 != 0.0 {
            // it_802B64FC: the homing turn.
            x8 = self.missile_homing(k, &it, x8, &at)?;
            let mut v = melee_runtime::mtx::Vec3 { x: at[3] * it.facing(), y: 0.0, z: 0.0 };
            melee_runtime::lbvector::rotate(&mut v, 4, x8 * -it.facing());
            vel.x = v.x;
            vel.y = v.y;
        } else if melee_core::fp::fmadds(vel.x, vel.x, vel.y * vel.y) > at[5] * at[5] {
            vel.x *= at[4];
            vel.y *= at[4];
        }
        it.vel = vel;
        // The item physics: position += velocity (Item_802697D4).
        it.pos.x += vel.x;
        it.pos.y += vel.y;
        it.pos.z += vel.z;
        // (The item-vs-stage collision, it_8026E71C, finds nothing in open air.)
        self.joint_mtx.insert(jref, translation_mtx(it.pos));
        // s_link 11: the hit capsule follows (x58 = the last position, state 3).
        let x58n = self.items[k].pos;
        it.class = ItemClass::SsMissile { owner, smash, life, x8, v0, v1, v3, born, fresh: false, facing, jref, a, hb: 3, x58: x58n };
        self.items[k] = it;
        Ok(false)
    }

    fn missile_release_smoke(&mut self, jref: JObjRef) {
        let mut host = Host { rand: &mut self.rand, joint_mtx: &self.joint_mtx, appsrt_hook_lists: &self.appsrt_hook_lists };
        self.ps.gen_release_jobj(&mut host, Some(jref));
    }

    /// `it_802B701C`: the explosion state.
    fn missile_explode(&mut self, k: usize, life: f32, x8: f32, v0: f32, v1: f32) -> Result<bool, String> {
        let _ = life;
        let mut it = self.items[k];
        let ItemClass::SsMissile { owner, smash, v3, born, facing, jref, a, x58, .. } = it.class else { unreachable!() };
        // efLib_DestroyAll(gobj), it_80272B40: efSync_Spawn(0x40C) -> efLib_CreateGenerator(0x19, &pos).
        self.missile_release_smoke(jref);
        let mut host = Host { rand: &mut self.rand, joint_mtx: &self.joint_mtx, appsrt_hook_lists: &self.appsrt_hook_lists };
        if let Some(g) = self.ps.gen_create(&mut host, 0, 0, 0x19) {
            self.ps.gen_mut(g).pos = it.pos;
        }
        // it_80273454 (zero velocity), it_8027518C (the lifetime), Item_80268E5C(gobj, 2).
        it.vel = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        it.msid = 2;
        it.class = ItemClass::SsMissile { owner, smash, life: EXPLOSION_LIFE, x8, v0, v1, v3, born, fresh: false, facing, jref, a, hb: 0, x58 };
        self.items[k] = it;
        Ok(false)
    }

    /// `it_802B64FC`: the homing turn rate. Returns the new `x8`.
    fn missile_homing(&mut self, k: usize, it: &SimItem, mut x8: f32, at: &[f32; 16]) -> Result<f32, String> {
        let ItemClass::SsMissile { owner, .. } = it.class else { unreachable!() };
        let facing = it.facing();
        // it_8026B634 -> ftLib_80086368: the closest opponent on the facing side.
        let mut best: Option<(f32, Vec3)> = None;
        let owner_id = self.players[owner].fp.player_id;
        for q in self.players.iter_mut() {
            if q.fp.player_id == owner_id || q.fp.x221F_b3() != 0 {
                continue;
            }
            let bone = q.fp.co_attrs.camera_zoom_target_bone as usize;
            let o = q.fp.co_attrs.x170;
            let j = q.skel.part(bone);
            let v = q.skel.world_point(j, Vec3 { x: o.x, y: o.y, z: o.z });
            if (facing == -1.0 && v.x > it.pos.x) || (facing == 1.0 && v.x < it.pos.x) {
                continue;
            }
            let (dx, dy) = (it.pos.x - v.x, it.pos.y - v.y);
            let diff = melee_core::fp::fmadds(dx, dx, dy * dy);
            if best.map_or(true, |(d, _)| diff < d) {
                best = Some((diff, v));
            }
        }
        let target = match best {
            Some((_, v)) => v,
            None => {
                // it_8026C258: item targets (Shy Guys & co.) are not modelled.
                if self.items.iter().any(|i| !matches!(i.class, ItemClass::SsMissile { .. } | ItemClass::FoxLaser { .. } | ItemClass::FoxGun { .. })) {
                    return Err("it_8026C258: lock-on item targets are not ported".into());
                }
                return Ok(x8);
            }
        };
        let _ = k;
        if target.x == 0.0 && target.y == 0.0 {
            return Ok(x8);
        }
        let mut vec2 = melee_runtime::mtx::Vec3 { x: it.vel.x, y: it.vel.y, z: 0.0 };
        melee_runtime::lbvector::normalize_xy(&mut vec2);
        let mut vec1 = melee_runtime::mtx::Vec3 { x: target.x - it.pos.x, y: target.y - it.pos.y, z: 0.0 };
        melee_runtime::lbvector::normalize_xy(&mut vec1);
        let ang = melee_runtime::lbvector::angle_xy(&vec2, &vec1);
        if ang < at[8] {
            return Ok(x8);
        }
        let mut vec0 = vec1;
        melee_runtime::lbvector::sub(&mut vec0, &vec2);
        if vec0.y > 0.001 {
            x8 -= at[6];
        } else if vec0.y < 0.001 {
            x8 += at[6];
        }
        // itSamusMissile_ClampTurn.
        let max = at[7];
        if x8.abs() > max {
            x8 = if x8 > 0.0 { max } else { -max };
        }
        Ok(x8)
    }
}

fn translation_mtx(p: Vec3) -> melee_runtime::mtx::Mtx {
    let mut m = melee_runtime::mtx::identity();
    m[0][3] = p.x;
    m[1][3] = p.y;
    m[2][3] = p.z;
    m
}

impl SimItem {
    /// The direction the item flies (`ip->facing_dir`).
    fn facing(&self) -> f32 {
        match self.class {
            ItemClass::SsMissile { facing, .. } => facing,
            _ => 1.0,
        }
    }
}
