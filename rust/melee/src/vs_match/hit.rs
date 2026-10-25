//! Item hitboxes on fighters: the part of `Fighter_8006CB94` (s_link 13) and `Fighter_ProcessHit_8006D1EC`
//! (s_link 14) a Fox / Falco laser reaches.
//!
//! The detection is `ftColl_8007925C` for an item with plain hitboxes (`ftColl_80077C60`, the damage log
//! `dmg_log0`), `ftColl_8007AB48` -> `ftColl_8007A06C` (the knockback, the hit effect) and, at s_link 14,
//! the damage commit (`Fighter_UnkTakeDamage_8006CC30`). A hit that carries knockback needs the
//! damage states (`ftCo_DamageN`..) and the launch, which are not ported: it returns `Err`. So does
//! everything else that is not reached by a laser hitting a standing, unshielded fighter: reflectors,
//! absorbers, shields, clanks with the victim's own hitboxes, tip hits, intangibility.
//!
//! The lasers' hit capsules are [`fox::LaserHits`]; their constant fields (flags, knockback, severity) are
//! what the C build's `it_802790C0` (the article script's `create_hitbox`) leaves in them.

use super::fox::{ItemClass, LaserHits, LASER_HIT_SCALE};
use super::*;
use melee_types::{HitCapsule, HSD_GObj, HSD_JObj, HitCapsule_Disabled, HurtCapsule_Enabled};

/// `DmgLogEntry` (`x0` = 2: an item's hit).
#[derive(Clone, Copy)]
struct LogEntry {
    /// Index of the item in `VsMatch::items`.
    item: usize,
    kb_growth: u32,
    wdsk: u32,
    bkb: u32,
    kb_angle: i32,
    element: u32,
    sfx_severity: i32,
    /// `entry->pos`: the hit capsule's `hurt_coll_pos`.
    pos: Vec3,
    /// `entry->x20`: `hit->damage`.
    damage: f32,
    /// `entry->size_of_xC`: the raw damage as a count (`unk_count`).
    unk_count: u32,
    /// `entry->hurt1->height` (the hurtbox that was hit: 0 high, 1 mid, 2 low).
    hurt_height: i32,
}

/// The laser's per-box constants, by item kind: (knockback growth, weight-dependent set knockback,
/// base knockback). Fox's shot has none (a damage-only hit); Falco's has set knockback.
fn laser_kb(kind: i32) -> (u32, u32, u32) {
    if kind == melee_types::It_Kind_Fox_Laser as i32 {
        (0, 0, 0)
    } else {
        (100, 5, 0)
    }
}

/// `HitElement_Normal`.
const ELEMENT: u32 = 0;
/// The laser's `kb_angle` (361: the Sakurai angle).
const KB_ANGLE: i32 = 361;
/// `hit->sfx_severity`.
const SEVERITY: i32 = 1;

impl VsMatch {
    /// The fighters' s_link 13: `ftColl_8007925C` + `ftColl_8007AB48` for each fighter.
    pub(super) fn item_hits(&mut self) -> Result<(), String> {
        for pi in 0..self.players.len() {
            {
                let p = &self.players[pi];
                if p.follower || p.fp.x221F_b3() != 0 || p.fp.x2219_b1() != 0 {
                    continue;
                }
            }
            // ftColl_800765E0: both damage logs are empty.
            let mut log: Vec<LogEntry> = Vec::new();
            let mut refreshed = false;
            for k in 0..self.items.len() {
                let it = self.items[k];
                let ItemClass::FoxLaser { owner, hits, .. } = it.class else { continue };
                if !hits.valid {
                    return Err("a laser made by the s_link 9 accessory: its hit state is not modelled".into());
                }
                // ftLib_80086FD4(gobj, item->owner): the shots of the fighter's own player pass through.
                if self.players[owner].fp.player_id == self.players[pi].fp.player_id {
                    continue;
                }
                // The victim's hurt capsules, as the skeleton stands at s_link 13 (lbColl_800083C4).
                if !refreshed {
                    hurt_positions(&mut self.players[pi]);
                    refreshed = true;
                }
                if let Some(e) = self.laser_vs_fighter(pi, k, &it, &hits)? {
                    log.push(e);
                }
            }
            if !log.is_empty() {
                self.damage_log0(pi, &log)?;
            }
        }
        Ok(())
    }

    /// `ftColl_8007925C`'s body for one laser against fighter `pi`.
    fn laser_vs_fighter(&mut self, pi: usize, k: usize, it: &super::fox::SimItem, hits: &LaserHits) -> Result<Option<LogEntry>, String> {
        let kind = it.kind;
        let cd = melee_fighter::hooks::common_data();
        // The victim's own live hitboxes would make `hit_count` nonzero: the clank test.
        {
            let fp = &self.players[pi].fp;
            if fp.anon_221b.x221B_b5() == 0 && fp.x914.iter().any(|h| h.state != HitCapsule_Disabled as _) {
                return Err("ftColl_8007925C: a shot against a fighter with live hitboxes (clank) is not ported".into());
            }
        }
        for j in 0..hits.n {
            if hits.state[j] == 0 {
                continue;
            }
            // lbColl_8000ACFC(fp, hurt): this shot already hit the fighter.
            if hits.victims & (1 << pi) != 0 {
                continue;
            }
            let fp = &mut self.players[pi].fp;
            // x42_b4 (set): a reflector or an absorber would catch the shot.
            if fp.reflecting() != 0 {
                return Err("ftColl_8007925C: a reflector against a shot is not ported".into());
            }
            if fp.x2218_b6() != 0 {
                return Err("ftColl_8007925C: an absorber against a shot is not ported".into());
            }
            // x42_b1 (set) with the fighter shielding.
            if fp.anon_221b.x221B_b0() != 0 {
                return Err("ftColl_8007925C: a shot against a shield is not ported".into());
            }
            if fp.x1988 == 2 || fp.x198C == 2 {
                continue;
            }
            // The hit capsule as the C holds it.
            let mut hit: HitCapsule = HitCapsule::zeroed();
            hit.x4C = melee_types::Vec { x: hits.x4c[j].x, y: hits.x4c[j].y, z: hits.x4c[j].z };
            hit.x58 = melee_types::Vec { x: hits.x58[j].x, y: hits.x58[j].y, z: hits.x58[j].z };
            hit.scale = LASER_HIT_SCALE;
            hit.damage = hits.dmg;
            // ftCommon_8007F804: the fighter's matrix only when its Z scale is not 1.
            if fp.x34_scale.z != 1.0 {
                return Err("ftColl_8007925C: a fighter with a Z scale is not ported".into());
            }
            let (scale_y, pos_z) = (fp.x34_scale.y, fp.cur_pos.z);
            let n_caps = fp.hurt_capsules_len as usize;
            for n in 0..n_caps {
                // hurt->x42_b6 is clear: the grabbable test is skipped.
                let hit_now = unsafe { melee_runtime::tu::lbcollision::lbColl_8000805C(&mut hit, &mut fp.hurt_capsules[n].capsule, core::ptr::null(), 0, 1.0, scale_y, pos_z) };
                let cap_state = fp.hurt_capsules[n].capsule.state;
                if !hit_now {
                    continue;
                }
                // ftColl_80077C60.
                if hit.coll_distance < cd.x7A8 {
                    return Err("ftColl_80077C60: a tip hit (inlineB1) is not ported".into());
                }
                let raw = hits.dmg;
                let scaled = raw * fp.dmg.x182c_behavior;
                if fp.x1988 != 0 || fp.x198C != 0 || fp.x221D_b6() != 0 || cap_state != HurtCapsule_Enabled as _ {
                    // The invincible / intangible branch: a sound only, and no log.
                    return Err("ftColl_80077C60: a hit on an invincible fighter is not ported".into());
                }
                let dmg_count: i32 = if scaled != 0.0 { if scaled as i32 != 0 { scaled as i32 } else { 1 } } else { 0 };
                if fp.x221C_b4() != 0 {
                    return Err("ftColl_80077C60: armor (x221C_b4) is not ported".into());
                }
                fp.dmg.x1838_percentTemp += scaled;
                if dmg_count > fp.dmg.x183C_applied {
                    fp.dmg.x183C_applied = dmg_count;
                }
                let (g, w, b) = laser_kb(kind);
                let entry = LogEntry {
                    item: k,
                    kb_growth: g,
                    wdsk: w,
                    bkb: b,
                    kb_angle: KB_ANGLE,
                    element: ELEMENT,
                    sfx_severity: SEVERITY,
                    pos: Vec3 { x: hit.hurt_coll_pos.x, y: hit.hurt_coll_pos.y, z: hit.hurt_coll_pos.z },
                    damage: hits.dmg,
                    unk_count: raw as u32,
                    hurt_height: fp.hurt_capsules[n].height as i32,
                };
                // item->xC34_damageDealt, and it_8026FAC4: the shot has hit this fighter.
                if let ItemClass::FoxLaser { hits: h, .. } = &mut self.items[k].class {
                    h.victims |= 1 << pi;
                    h.dealt = true;
                }
                // ftColl_80078998: plStale_UpdateStaleMovesFromItem and the combo counter.
                let owner = if let ItemClass::FoxLaser { owner, hits, .. } = self.items[k].class { Some((owner, hits.atk_id, hits.atk_inst)) } else { None };
                if let Some((o, id, inst)) = owner {
                    let of = &self.players[o].fp;
                    if of.player_id != self.players[pi].fp.player_id {
                        melee_fighter::stale::update_stale_moves_from_item(Some(of), id, inst);
                    }
                    self.combo_count(o, pi, id)?;
                }
                return Ok(Some(entry));
            }
        }
        Ok(None)
    }

    /// `ftColl_8007646C` -> `ftColl_800763C0(owner, victim, attack_id)`: the owner's combo counter.
    fn combo_count(&mut self, owner: usize, victim: usize, attack_id: i32) -> Result<(), String> {
        if owner == victim {
            return Ok(());
        }
        let cd = melee_fighter::hooks::common_data();
        // The victim's identity: its fighter record (the C compares the GObj pointers).
        let v: *mut HSD_GObj = &mut *self.players[victim].fp as *mut Fighter as *mut HSD_GObj;
        let fp = &mut self.players[owner].fp;
        if fp.x2094.is_null() {
            fp.x208C = attack_id;
            fp.x2090 = 1;
            fp.x2094 = v;
        } else if fp.x2094 == v {
            if attack_id != 1 && fp.x208C == attack_id {
                fp.x2090 += 1;
                if fp.x2090 as i32 >= cd.x4C4 as i32 {
                    return Err("ftColl_800763C0: a combo long enough to push the victim (x2092) is not ported".into());
                }
            } else {
                fp.x2090 = 0;
                fp.x208C = attack_id;
            }
        }
        Ok(())
    }

    /// `ftColl_8007AB48` -> `ftColl_8007A06C(gobj, &fp->dmg.facing_dir_1, dmg_log0, n, 1)`: the strongest
    /// entry's knockback, the hit effect and `fp->dmg`.
    fn damage_log0(&mut self, pi: usize, log: &[LogEntry]) -> Result<(), String> {
        let mut best_kb = -1.0f32;
        let mut best = 0usize;
        for (i, e) in log.iter().enumerate() {
            let (owner, _) = match self.items[e.item].class {
                ItemClass::FoxLaser { owner, hits, .. } => (owner, hits),
                _ => return Err("damage log of an item that is not a laser".into()),
            };
            // Player_GetAttackRatio(owner) / Player_GetDefenseRatio / gm_8016B248: 1 in these matches.
            let fp = &self.players[pi].fp;
            let attack = 1.0f32;
            let defense = 1.0f32;
            let stage = 1.0f32;
            let _ = owner;
            // ftColl_80079AB0's formula (A06C inlines it for an item entry).
            let mut h: HitCapsule = HitCapsule::zeroed();
            h.x24 = e.kb_growth;
            h.x28 = e.wdsk;
            h.x2C = e.bkb;
            let kb = melee_fighter::tu::ftcoll::ftcoll_80079ab0(fp, &h, e.unk_count, stage, attack, defense, fp.co_attrs.weight);
            // The hit effect (arg4 != 0), per entry in log order.
            self.hit_effect(pi, e, kb)?;
            let fp = &self.players[pi].fp;
            let _ = fp;
            if kb > best_kb {
                best_kb = kb;
                best = i;
            }
        }
        let e = log[best];
        // The result struct (fp->dmg): dir, angle, kb, ...
        let (item_pos_x, vel_x) = (self.items[e.item].pos.x, self.items[e.item].vel.x);
        let fp = &mut self.players[pi].fp;
        let mut ax = vel_x;
        if ax < 0.0 {
            ax = -ax;
        }
        let dir = if ax < item_common_x78() {
            if fp.cur_pos.x > item_pos_x { -1.0 } else { 1.0 }
        } else if vel_x < 0.0 {
            1.0
        } else {
            -1.0
        };
        fp.dmg.facing_dir_1 = dir;
        fp.dmg.x1848_kb_angle = e.kb_angle;
        fp.dmg.x184c_damaged_hurtbox = e.hurt_height;
        fp.dmg.kb_applied = best_kb;
        fp.dmg.x1854_collpos = melee_types::Vec { x: e.pos.x, y: e.pos.y, z: e.pos.z };
        fp.dmg.x1860_element = e.element as _;
        fp.dmg.x1864 = e.sfx_severity;
        fp.dmg.x1868_source = std::ptr::null_mut();
        fp.dmg.x186c = e.damage.to_bits() as i32;
        // ftColl_8007861C(owner, gobj, 2, item kind, ...): the attacker's identity on the victim.
        let (owner, ikind) = match self.items[e.item].class {
            ItemClass::FoxLaser { owner, .. } => (owner, self.items[e.item].kind),
            _ => unreachable!(),
        };
        let (spawn, ply, sub) = {
            let o = &self.players[owner].fp;
            (o.x8_spawnNum, o.player_id, o.is_sub_fighter())
        };
        let fp = &mut self.players[pi].fp;
        fp.dmg.x18C0 = if owner == pi { 0 } else { spawn as _ };
        fp.dmg.x18c4_source_ply = ply as _;
        fp.dmg.x18C8 = -1;
        fp.set_x221F_b5(sub as _);
        fp.dmg.x18CC = 2;
        fp.dmg.x18D0 = ikind;
        Ok(())
    }

    /// `ftColl_80078538` (via `hit_effect_ids[HitElement_Normal]` = `Ef_Id_Unk1000`): the hit spark.
    fn hit_effect(&mut self, pi: usize, e: &LogEntry, kb: f32) -> Result<(), String> {
        let cd = melee_fighter::hooks::common_data();
        if kb >= cd.x3F0 {
            return Err("ftColl_80078538: the big hit spark (0x3F3) is not ported".into());
        }
        // efSync_Spawn(1000, 0, pos, &u_dmg) -> efAsync_Dispatch(0x3E8): Randi(8) picks effect 9 or 10, scaled
        // by 0.04 * damage + 0.3 (clamped to 0.3..1.5).
        let u_dmg = e.damage as u32 as f32;
        let id = if self.rand.randi(8) == 0 { 9 } else { 10 };
        let mut s = 0.04f32 * u_dmg + 0.3f32;
        if s < 0.3 {
            s = 0.3;
        }
        if s > 1.5 {
            s = 1.5;
        }
        self.ef_create_rot(id, pi, s, Some([e.pos.x, e.pos.y, e.pos.z]), None, None)?;
        // dmg (the severity) >= 1: the extra spark.
        if e.sfx_severity >= 1 {
            match self.players[pi].fp.co_attrs.hit_spark_variant {
                0 => {
                    if self.rand.randi(cd.x3F4 as i32) == 0 {
                        return Err("ftColl_80078538: the extra hit spark (efSync_Spawn 0x3EF) is not ported".into());
                    }
                }
                1 => {
                    let _ = self.rand.randi(cd.x3F8 as i32);
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// `Fighter_ProcessHit_8006D1EC` after the shield upkeep: the damage commit, the knockback branch (the
    /// damage states), the hitlag, and the field resets. Branches other than a laser's hit return `Err`.
    pub(super) fn process_hit_damage(&mut self, pi: usize) -> Result<(), String> {
        {
            let fp = &self.players[pi].fp;
            if fp.dmg.x189C_unk_num_frames > 0.0 {
                return Err("Fighter_ProcessHit_8006D1EC: x189C (ftColl_8007BE3C) is not ported".into());
            }
            if fp.dmg.x18a0 != 0.0 || fp.x19A4 != 0 || fp.dmg.int_value != 0 || fp.dmg.x1914 != 0 || fp.dmg.x1924 != 0 {
                return Err("Fighter_ProcessHit_8006D1EC: a shield / dealt-damage / reflect branch is not ported".into());
            }
        }
        let msid = self.players[pi].fp.motion_id as i32;
        let force = self.players[pi].fp.dmg.kb_applied;
        let mut b1: i32 = 0;
        let mut b2 = false;
        if force != 0.0 {
            let ga = self.players[pi].fp.ground_or_air as i32;
            let g: *mut HSD_GObj = &mut *self.players[pi].gobj;
            {
                let fp = &mut self.players[pi].fp;
                fp.dmg.x189C_unk_num_frames = 0.0;
                let d = fp.dmg.x1838_percentTemp;
                take_damage(fp, d)?;
                melee_fighter::tu::ftCo_Damage::ftCo_Damage_CalcKnockback(fp);
                if fp.kind as i32 == melee_types::Ft_Kind_Kirby as i32 {
                    return Err("ftKb_SpecialN_800F5BA4 (Kirby) is not ported".into());
                }
                if fp.take_dmg_2_cb.is_some() {
                    return Err("take_dmg_2_cb is not ported".into());
                }
                if fp.no_kb() != 0 {
                    return Err("no_kb (Master / Crazy Hand) is not ported".into());
                }
                if fp.x1828 != 0 {
                    return Err("x1828 != 0 (a hit while grabbed) is not ported".into());
                }
            }
            self.with_sim(|_| melee_fighter::tu::ftCo_Damage::ftCo_8008EC90(g));
            self.take_dmg_err()?;
            let fp = &mut self.players[pi].fp;
            let applied = fp.dmg.x183C_applied;
            b2 = true;
            melee_fighter::ftdmgfall::ftco_80090594(fp, fp.dmg.x1860_element as i32, applied, msid, ga, fp.x1960_vibrateMult);
            let d = fp.dmg.x1838_percentTemp;
            melee_fighter::ftcommon2::ftcommon_8007ed50(fp, d as i32);
            b1 = applied;
        } else if self.players[pi].fp.dmg.x1838_percentTemp != 0.0 {
            // (the no-knockback hit: Fighter_UnkTakeDamage_8006CC30, ftKb_SpecialN_800F5C34 (Kirby only),
            // ftCommon_800804FC)
            let fp = &mut self.players[pi].fp;
            let d = fp.dmg.x1838_percentTemp;
            take_damage(fp, d)?;
            melee_fighter::ftcommon2::ftcommon_800804fc(fp);
        }
        let cd = melee_fighter::hooks::common_data();
        let fp = &mut self.players[pi].fp;
        if fp.x2224_b3() != 0 {
            return Err("ftCo_800C8D00 (x2224_b3) is not ported".into());
        }
        if b1 != 0 {
            fp.dmg.x195c_hitlag_frames = melee_fighter::ftcommon::calc_hitlag(b1, msid, fp.x1960_vibrateMult);
            if fp.dmg.x195c_hitlag_frames < fp.x1964 {
                fp.dmg.x195c_hitlag_frames = fp.x1964;
            }
            if fp.dmg.x195c_hitlag_frames > 0.0 {
                if fp.dmg.x195c_hitlag_frames > cd.x194_unkHitLagFrames {
                    fp.dmg.x195c_hitlag_frames = cd.x194_unkHitLagFrames;
                }
                fp.set_allow_sdi(1);
                if b2 {
                    fp.set_x221A_b3(1);
                }
                if fp.x2219_b5() == 0 {
                    enter_hitlag(fp)?;
                }
            }
        } else {
            melee_fighter::ftdmgfall::ftco_80090718(fp);
        }
        if fp.x221A_b0() != 0 || fp.dmg.x1958 != 0.0 {
            return Err("Fighter_ProcessHit_8006D1EC: x221A_b0 / x1958 (a delayed hitlag) is not ported".into());
        }
        if fp.dmg.x1928 != 0.0 {
            return Err("Fighter_ProcessHit_8006D1EC: x1928 (an attacker shield push) is not ported".into());
        }
        fp.dmg.x1838_percentTemp = 0.0;
        fp.dmg.x183C_applied = 0;
        fp.x1828 = 0;
        fp.dmg.kb_applied = 0.0;
        fp.dmg.x18a0 = 0.0;
        fp.dmg.x1840 = 0;
        fp.dmg.x1914 = 0;
        fp.dmg.int_value = 0;
        fp.dmg.x191C = 0.0;
        fp.unk_gobj = std::ptr::null_mut();
        fp.set_x221C_b5(0);
        fp.dmg.x1924 = 0;
        fp.dmg.x1928 = 0.0;
        fp.x19A0_shieldDamageTaken = 0;
        fp.x19A4 = 0;
        fp.x19A8 = std::ptr::null_mut();
        fp.ReflectAttr.x1A3C_damageOver = 0;
        fp.ReflectAttr.x1A2C_reflectHitDirection = 0.0;
        fp.AbsorbAttr.x1A40_absorbHitDirection = 0.0;
        fp.AbsorbAttr.x1A44_damageTaken = 0;
        fp.AbsorbAttr.x1A48_hitsTaken = 0;
        fp.x1960_vibrateMult = 1.0;
        fp.x1964 = 0.0;
        fp.dmg.x1950 = 0;
        if (fp.x2219_b6() == 0 || fp.dmg.x18F4 != 0) && fp.x2100 != -1 {
            return Err("ftCo_800C2FD8 (a sword trail) is not ported".into());
        }
        Ok(())
    }

    /// The items' s_link 14 for the lasers: a shot that dealt damage is destroyed.
    pub(super) fn lasers_destroy_dealt(&mut self) {
        self.items.retain(|it| !matches!(it.class, ItemClass::FoxLaser { hits, .. } if hits.dealt));
    }
}

/// `it_804D6D28->x78_float` (ItCo.dat): the speed under which an item's horizontal knock direction comes from
/// positions rather than velocity.
fn item_common_x78() -> f32 {
    // Not read from ItCo yet: any shot moves faster than it.
    0.0
}

/// `lbColl_800083C4` for each of the fighter's hurt capsules, from the skeleton as it stands.
fn hurt_positions(p: &mut Player) {
    let n = p.hurtboxes.len().min(p.fp.hurt_capsules.len());
    p.fp.hurt_capsules_len = n as _;
    // The bone pointers are never followed (the capsules are marked updated); the matrix argument of the
    // collision test just needs a valid JObj.
    thread_local! {
        static DUMMY: *mut HSD_JObj = Box::leak(Box::new(HSD_JObj::zeroed()));
    }
    let dummy = DUMMY.with(|d| *d);
    for i in 0..n {
        let (part, a, b, scale) = p.hurtboxes[i];
        let j = p.skel.part(part as usize);
        let pa = p.skel.world_point(j, Vec3 { x: a[0], y: a[1], z: a[2] });
        let pb = p.skel.world_point(j, Vec3 { x: b[0], y: b[1], z: b[2] });
        let c = &mut p.fp.hurt_capsules[i].capsule;
        c.a_pos = melee_types::Vec { x: pa.x, y: pa.y, z: pa.z };
        c.b_pos = melee_types::Vec { x: pb.x, y: pb.y, z: pb.z };
        c.scale = scale;
        c.bone = dummy;
        c.set_skip_update_pos(1);
    }
}

/// `IfDamageState`: the fields of the HUD's percent display that decide its RNG draws.
#[derive(Clone, Copy, Debug)]
pub struct HudDamage {
    /// `damage_percent` / `old_damage` (-1: not read yet).
    dmg: i32,
    old: i32,
    /// `frames_of_shake_remaining`.
    shake: u8,
}

impl Default for HudDamage {
    fn default() -> Self {
        HudDamage { dmg: -1, old: -1, shake: 0 }
    }
}

impl HudDamage {
    /// One frame of `ifStatus_802F5B48` then `ifStatus_802F4EDC`. `percent` is `Player_GetDamage`;
    /// `explode` the death animation's first run (`ifStatus_PercentOnDeathAnimationThink`: two draws per
    /// digit, four digits).
    pub fn step(&mut self, percent: f32, explode: bool, randf: &mut dyn FnMut() -> f32) {
        // ifStatus_802F5B48 (not a stamina match).
        self.old = self.dmg;
        self.dmg = (percent as i32).clamp(0, 0x3E7);
        let force_shake = self.old != -1 && self.dmg > self.old;
        // ifStatus_802F4EDC.
        if explode {
            for _ in 0..8 {
                randf();
            }
            return;
        }
        if force_shake {
            self.shake = 10;
        }
        // ifStatus_802F4B84: eight draws a frame (an x and a y offset for each of the four digits) while
        // the shake runs; its last frame (1) only puts the digits back.
        if self.shake != 0 {
            if self.shake == 1 {
                self.shake = 0;
                return;
            }
            for _ in 0..8 {
                randf();
            }
            self.shake -= 1;
        }
    }
}

/// `Fighter_UnkTakeDamage_8006CC30` -> `Fighter_TakeDamage_8006CC7C` (the percent; no metal / stamina).
fn take_damage(fp: &mut Fighter, d: f32) -> Result<(), String> {
    if fp.x2226_b4() == 0 || fp.x2226_b3() != 0 {
        fp.dmg.x1830_percent += d;
        if fp.metal_timer != 0 {
            return Err("Fighter_TakeDamage_8006CC7C: a metal fighter is not ported".into());
        }
        if fp.x2034 != 0 {
            return Err("Fighter_TakeDamage_8006CC7C: x2034 is not ported".into());
        }
        if fp.dmg.x1830_percent > 999.0 {
            fp.dmg.x1830_percent = 999.0;
        }
    }
    // ftCommon_8007EA90 (rumble) and ftCo_800C8C84.
    // ftCommon_800804FC is the caller's for a hit without knockback.
    Ok(())
}

/// `Fighter_UnkRecursiveFunc_8006D044` for a fighter without a partner.
fn enter_hitlag(fp: &mut Fighter) -> Result<(), String> {
    if let Some(cb) = fp.pre_hitlag_cb {
        let g = fp.gobj;
        unsafe { cb(g) };
    }
    fp.set_x2219_b5(1);
    if !fp.x1A5C.is_null() {
        return Err("Fighter_UnkRecursiveFunc_8006D044: a partner (x1A5C) is not ported".into());
    }
    Ok(())
}
