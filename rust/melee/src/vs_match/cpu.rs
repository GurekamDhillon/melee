//! The CPU controller (`src/melee/ft/kinds/ftCommon/ftCo_0A01.c`, `ftcpuattack.c`): `Fighter_8006ABA0`
//! (s_link 2) runs `ftCo_800B3900` for every CPU-controlled fighter. The AI senses the stage,
//! picks a behaviour (`cpu.x18`) and writes a command script that `ftCo_800B3E04`
//! (`melee_fighter::tu::ftcmdscript`) turns into the stick / button values `fp.cpu` holds.
//!
//! Hand port, function by function, in the C's order. What the match has not reached yet returns
//! `Err("... not ported")` at the branch, never a guess (the first such stop is the next port).
//! FMA: the shipping build contracts `a*b + c` (see `tools/rust-port/fma_sites.py`); every fused
//! site is marked `FMA`. `sqrtf` is the MSL Newton `melee_runtime::gekko::sqrtf`.

use super::fox::SimItem;
use super::BlastZone;
use melee_fighter::tu::ftcmdscript as cs;
use melee_runtime::gekko::sqrtf;
use melee_runtime::random::HsdRand;
use melee_types::*;
use melee_world::mp::check::SegHit;
use melee_world::mp::lines::GroundColl;
use melee_world::vec::Vec3 as WVec3;
use melee_world::mp::types::LINE_FLAG_PLATFORM;

type R<T = ()> = Result<T, String>;

/// Everything the AI reads outside its own fighter.
pub(super) struct Cx<'a> {
    /// Every fighter of the match in `HSD_GObjPLinkHead[FIGHTER]` order.
    pub fps: std::vec::Vec<*mut Fighter>,
    pub ground: &'a mut GroundColl,
    pub rand: &'a mut HsdRand,
    pub items: &'a [SimItem],
    pub blast: BlastZone,
    /// `stage_info.grkind`.
    pub grkind: i32,
    /// `Fighter_804D64FC->x20[kind]`: the per-character distance thresholds.
    pub dist_thresh: &'a [f32],
    /// `p_ftCommonData->horizontal_stick_deadzone`.
    pub h_deadzone: f32,
    /// PlCo.dat and the offset of `Fighter_804D64FC` (`pData[22]`).
    pub plco: &'a [u8],
    pub tbl: u32,
}

#[inline]
fn b(v: u8) -> bool {
    v != 0
}

/// `f32` view of `a + b` where C promotes through double (`x - 5.0`): computed in double, stored f32.
#[inline]
fn dsub(a: f32, c: f64) -> f32 {
    (a as f64 - c) as f32
}
#[inline]
fn dadd(c: f64, a: f32) -> f32 {
    (c + a as f64) as f32
}

impl Cx<'_> {
    fn left(&self) -> f32 {
        self.blast.left
    }
    fn right(&self) -> f32 {
        self.blast.right
    }
    fn top(&self) -> f32 {
        self.blast.top
    }
    fn bottom(&self) -> f32 {
        self.blast.bottom
    }

    /// `mpCheckFloor(ax, ay, bx, by, y_offset, ..., -1, -1, -1, NULL, NULL)`.
    fn check_floor(&mut self, ax: f32, ay: f32, bx: f32, by: f32, y_offset: f32) -> Option<SegHit> {
        self.ground.mp_check_floor(ax, ay, bx, by, y_offset, -1, -1, -1, None)
    }

    /// `ftCo_800A1B38`: the line belongs to a stage hazard the AI ignores (Big Blue, Fountain's
    /// ..., Corneria, Venom). Only reachable on those stages.
    fn ignored_floor(&self, _line: i32) -> R<bool> {
        match self.grkind {
            g if g == Gr_Kind_BigBlue as i32 || g == Gr_Kind_Inishie1 as i32 || g == Gr_Kind_Corneria as i32 || g == Gr_Kind_Venom as i32 => {
                Err(format!("ftCo_800A1B38 on stage {g} not ported"))
            }
            _ => Ok(false),
        }
    }

    /// `ftCo_800A0FB0` with the (x, y) -> (x, y - 1000) probe the callers use: the floor point.
    fn probe(&mut self, ax: f32, ay: f32, bx: f32, by: f32, y_offset: f32) -> R<Option<SegHit>> {
        let hit = self.check_floor(ax, ay, bx, by, y_offset);
        if let Some(h) = &hit {
            if self.ignored_floor(h.line_id)? {
                return Ok(None);
            }
        }
        Ok(hit)
    }

    /// The `inlineD0`-style test of a point: outside the blast zone shrunk by the CPU's box.
    fn out_of_bounds(&self, c: &CpuFighter, x: f32, y: f32) -> bool {
        x < self.left() + c.half_width || x > self.right() - c.half_width || y < self.bottom() + c.half_height || y > self.top() - c.half_height
    }

    /// `grLib_801C9E60`: the stage's scroll velocity (zero off Rainbow Cruise / Big Blue / Ice Mt).
    fn stage_velocity(&self) -> R<Option<WVec3>> {
        let g = self.grkind;
        if g == Gr_Kind_RCruise as i32 || g == Gr_Kind_BigBlue as i32 || g == Gr_Kind_Icemt as i32 {
            return Err(format!("grLib_801C9E60 on stage {g} not ported"));
        }
        Ok(None)
    }
}

/// `ftCo_800A1AB4`: the distance between two fighters. FMA: `fma(dx, dx, dy * dy)`.
pub(super) fn ftco_800a1ab4(a: &Fighter, c: &Fighter) -> f32 {
    let dx = a.cur_pos.x - c.cur_pos.x;
    let dy = a.cur_pos.y - c.cur_pos.y;
    sqrtf(dx.mul_add(dx, dy * dy))
}

/// `ftCo_800A1C44`: the fighter cannot be targeted (hidden, captured, ...).
fn ftco_800a1c44(fp: &Fighter) -> bool {
    b(fp.x2219_b1()) || fp.x2164 != 0 || (fp.x2168 != 0 && fp.x2338.x == 0) || b(fp.x221F_b3())
}

/// `ftCo_800A0F00` (the `ft_80087A18` gate): the fighter is frozen / held by a hit-lag effect.
fn ftco_800a0f00(fp: &Fighter) -> R<bool> {
    if b(fp.x2226_b4()) {
        return Err("ft_80087A18 (x2226_b4: ftCo_800C06B4) not ported".into());
    }
    Ok(false)
}

/// `inlineD1`: the fighter is not a valid target / threat (inactive, dead, frozen).
fn inline_d1(fp: &Fighter) -> R<bool> {
    if b(fp.x221F_b3()) || b(fp.x2224_b2()) || ftco_800a0f00(fp)? {
        return Ok(true);
    }
    // ftLib_8008732C: DeadDown .. DeadUpFallHitCameraIce.
    Ok((ftCo_MS_DeadDown as i32..=ftCo_MS_DeadUpFallHitCameraIce as i32).contains(&fp.motion_id))
}

/// `ftCo_IsAlly`: free-for-all (`gm_8016B14C` true): allies only share a player id.
fn ftco_is_ally(a: &Fighter, c: &Fighter) -> bool {
    a.player_id == c.player_id
}

/// `ftCo_800A5980`: the fighter is in a state where the CPU treats it as grounded-helpless.
fn ftco_800a5980(fp: &Fighter) -> bool {
    ftco_800a5980_motion(fp.motion_id)
}

/// `ftCo_800A5980` on a motion id (`ftCo_800BB220` calls it with an `Item*`: the field at +0x10 is the
/// item's `kind`).
fn ftco_800a5980_motion(m: i32) -> bool {
    (m >= ftCo_MS_LandingFallSpecial as i32 && m < ftCo_MS_Attack100Start as i32)
        || m == ftCo_MS_Furafura as i32
        || m == ftCo_MS_Catch as i32
        || m == ftCo_MS_CatchPull as i32
        || m == ftCo_MS_CatchAttack as i32
}

/// `ftCo_800A5ACC`: the level-scaled reaction gate.
fn ftco_800a5acc(fp: &Fighter) -> bool {
    let d = &fp.cpu;
    if !b(d.xF9_b6()) {
        return false;
    }
    let t = d.x7C % 120;
    match d.level {
        2 => t > 0x64,
        3 => t > 0x50,
        4 => t > 0x3C,
        5 => t > 0x28,
        6 => t > 0x1E,
        7 => t > 0x14,
        8 => t > 0xA,
        9 => true,
        _ => false,
    }
}

/// `ftCo_800A1CA8`.
fn ftco_800a1ca8(fp: &Fighter) -> bool {
    fp.x2168 != 0
}

/// `ftCo_800A5CE0`: the closest valid enemy.
fn ftco_800a5ce0(cx: &Cx, me: *mut Fighter) -> R<*mut Fighter> {
    let fp = unsafe { &*me };
    if ftco_800a1c44(fp) {
        return Ok(core::ptr::null_mut());
    }
    let mut closest: *mut Fighter = core::ptr::null_mut();
    let mut best = 0.0f32;
    for &cur in &cx.fps {
        if cur == me {
            continue;
        }
        let o = unsafe { &*cur };
        if inline_d1(o)? {
            continue;
        }
        if !(b(o.x221D_b6() as u8) || (o.x2168 != 0 && o.x2338.x == 0)) {
            continue;
        }
        if ftco_is_ally(fp, o) {
            continue;
        }
        if cx.out_of_bounds(&fp.cpu, o.cur_pos.x, o.cur_pos.y) {
            continue;
        }
        if closest.is_null() {
            closest = cur;
            best = ftco_800a1ab4(fp, o);
        } else {
            let d = ftco_800a1ab4(fp, o);
            if best > d {
                best = d;
                closest = cur;
            }
        }
    }
    Ok(closest)
}

/// `ftCo_800A4BEC`: with no valid target, the nearest fighter, remembered for a random time
/// (`cpu.x48` while `xF9_b0`).
fn ftco_800a4bec(cx: &mut Cx, me: *mut Fighter) -> R<*mut Fighter> {
    let fp = unsafe { &mut *me };
    let mut closest: *mut Fighter = core::ptr::null_mut();
    let mut best = 0.0f32;
    for i in 0..cx.fps.len() {
        let cur = cx.fps[i];
        if cur == me {
            continue;
        }
        let o = unsafe { &*cur };
        if cx.out_of_bounds(&fp.cpu, o.cur_pos.x, o.cur_pos.y) || ftco_is_ally(fp, o) {
            continue;
        }
        if ftco_800a1c44(o) || inline_d1(o)? {
            continue;
        }
        if b(fp.cpu.xF9_b0()) {
            if cur as *mut core::ffi::c_void == fp.cpu.x48 {
                return Ok(cur);
            }
        } else if closest.is_null() {
            closest = cur;
            best = ftco_800a1ab4(fp, o);
        } else {
            let d = ftco_800a1ab4(fp, o);
            if best > d {
                best = d;
                closest = cur;
            }
        }
    }
    if closest.is_null() {
        fp.cpu.set_xF9_b0(0);
    } else {
        fp.cpu.set_xF9_b0(1);
        let r = cx.rand.randf();
        // 120.0f * (0.5f * (0.5f * r)), stored to an int.
        fp.cpu.x30 = (120.0f32 * (0.5f32 * (0.5f32 * r))) as i32;
        fp.cpu.x48 = closest as *mut core::ffi::c_void;
    }
    Ok(closest)
}

/// `ftCo_800A1F3C`: set the CPU's destination (stage-specific island hops need a table).
fn ftco_800a1f3c(cx: &Cx, fp: &mut Fighter, x: f32, y: f32, tol: f32) -> R {
    if fp.cpu.x60 == 0 {
        fp.cpu.x54.x = x;
        fp.cpu.x54.y = y;
        fp.cpu.x38 = tol;
        // ftCo_800A1CC4(fp, ftCo_803C6594[grkind]): only Great Bay (6) and Shrine (7) have lists.
        if cx.grkind == Gr_Kind_GreatBay as i32 || cx.grkind == Gr_Kind_Shrine as i32 {
            return Err("ftCo_800A1CC4 (stage island list) not ported".into());
        }
    }
    Ok(())
}

/// `ftCo_800A229C`: the camera / blast-zone emergency check. 0: nothing, -1: off the camera,
/// 1 / 2: stage-specific rescues.
fn ftco_800a229c(cx: &Cx, fp: &Fighter) -> R<i32> {
    let g = cx.grkind;
    if g == Gr_Kind_GreatBay as i32 || g == Gr_Kind_Castle as i32 || g == Gr_Kind_Zebes as i32 || g == Gr_Kind_RCruise as i32 || g == Gr_Kind_BigBlue as i32 || g == Gr_Kind_Icemt as i32 {
        return Err(format!("ftCo_800A229C on stage {g} not ported"));
    }
    // Camera_8003118C(&cur_pos, 0.0) == 0: outside the camera bounds.
    let slope = -10000.0f32 + 1.0;
    let bottom = if cx.blast.cam_bottom > slope { cx.blast.cam_bottom } else { slope };
    let p = &fp.cur_pos;
    let outside = p.x < cx.blast.cam_left - 0.0 || p.x > cx.blast.cam_right + 0.0 || p.y > cx.blast.cam_top + 0.0 || p.y < bottom - 0.0;
    Ok(if outside { -1 } else { 0 })
}

/// `ftCo_800B33B0`: the per-frame sensing.
fn ftco_800b33b0(cx: &mut Cx, fp: &mut Fighter) -> R {
    if std::env::var_os("MELEE_GOLDEN_CPUDBG").is_some() { eprintln!("B33B0 slot {} x7C {} xF9_b0 {}", fp.player_id, fp.cpu.x7C, fp.cpu.xF9_b0()); }
    if fp.cpu.x7C % 300 == 0 {
        let r = cx.rand.randf();
        // FMA: 0.04f * level + 0.3f
        let lim = 0.04f32.mul_add(fp.cpu.level as f32, 0.3f32);
        fp.cpu.set_xFA_b34(if r < lim { 1 } else { 0 });
    }
    let (fx, fy) = (fp.cur_pos.x, fp.cur_pos.y);
    // The floor under the fighter (the same probe, ignoring hazard lines).
    let hit = cx.probe(fx, 10.0f32 + fy, fx, dsub(fy, 1000.0), 0.0)?;
    let in_bounds = match hit {
        Some(h) => !cx.out_of_bounds(&fp.cpu, h.pos.x, h.pos.y),
        None => false,
    };
    fp.cpu.set_xFA_b5(in_bounds as u8);
    let vy = fp.coll_data.cur_pos.y - fp.coll_data.last_pos.y;
    let vx = fp.coll_data.cur_pos.x - fp.coll_data.last_pos.x;
    // FMA: fma(vx, vx, vy * vy)
    let vmag = sqrtf(vx.mul_add(vx, vy * vy));
    if (vmag as f64) < 0.01 {
        fp.cpu.x84 += 1;
    } else {
        fp.cpu.x84 = 0;
    }
    if b(fp.cpu.xF9_b0()) {
        if fp.cpu.x30 != 0 {
            fp.cpu.x30 -= 1;
        } else {
            fp.cpu.set_xF9_b0(0);
        }
    } else {
        let r = cx.rand.randf();
        fp.cpu.x30 = (120.0f32 * (0.5f32 * (0.5f32 * r))) as i32;
    }
    if fp.cpu.x60 != 0 {
        fp.cpu.x60 -= 1;
        if fp.cpu.x60 == 0 {
            fp.cpu.x54 = fp.cpu.x64;
        }
    }
    // The floor under the destination: a moving floor carries it.
    let (sx, sy) = (fp.cpu.x54.x, fp.cpu.x54.y);
    let t = cx.probe(sx, dadd(2.0, sy), sx, dsub(sy, 2.0), 0.0)?;
    if let Some(h) = t {
        if let Some(v) = cx.ground.mp_get_speed(h.line_id, WVec3 { x: h.pos.x, y: h.pos.y, z: h.pos.z }) {
            fp.cpu.x54.x += v.x;
            fp.cpu.x54.y += v.y;
        }
    } else if let Some(v) = cx.stage_velocity()? {
        fp.cpu.x54.x += v.x;
        fp.cpu.x54.y += v.y;
    }
    // ftCo_CpuUpdateRecoveryScale
    if fp.cpu.x7C % 30 == 0 {
        let r = cx.rand.randf();
        // FMA (f64): 0.05 * (level + 1) + 0.05 * rand
        fp.cpu.x570 = (0.05f64.mul_add((fp.cpu.level + 1) as f64, 0.05f64 * r as f64)) as f32;
    }
    ftco_800a0cb0(cx, fp);
    // The ceiling probe (xFA_b6: under a ceiling, Corneria only).
    if b(fp.cpu.xFA_b6()) {
        let (x, y) = (fp.cur_pos.x, fp.cur_pos.y);
        if cx.ground.mp_check_ceiling(x, y, x, 1000.0f32 + y, -1, -1).is_none() {
            fp.cpu.set_xFA_b6(0);
        }
    } else if fp.ground_or_air == GA_Ground as _ && cx.grkind == Gr_Kind_Corneria as i32 {
        return Err("grCorneria_801E2E50 not ported".into());
    }
    Ok(())
}

/// `ftCo_800A0CB0`: the recovery distance scale, re-rolled every 600 frames.
fn ftco_800a0cb0(cx: &mut Cx, fp: &mut Fighter) {
    if fp.cpu.x7C % 600 == 0 {
        let x = cx.rand.randf();
        let x = x * x * x;
        let rand = 1.0f32 - x;
        let kind = fp.kind;
        fp.cpu.x56C = if kind == Ft_Kind_Donkey || kind == Ft_Kind_Koopa {
            (9.0f64 * rand as f64) as f32
        } else if kind == Ft_Kind_GKoops {
            (18.0f64 * rand as f64) as f32
        } else if kind == Ft_Kind_Nana {
            (1.0f64 * rand as f64) as f32
        } else {
            (4.0f64 * rand as f64) as f32
        };
    }
}

/// `ftCo_800B3900`.
pub(super) fn ftco_800b3900(cx: &mut Cx, me: *mut Fighter) -> R {
    let fp = unsafe { &mut *me };
    ftco_800b33b0(cx, fp)?;
    ftco_800b2afc(cx, me)?;
    ftco_800b2790(cx, me)?;
    cs::ftco_800b3e04(fp);
    ftco_800b0af4(cx, fp)?;
    fp.cpu.x7C += 1;
    Ok(())
}

/// `ftCo_800B0AF4`: Nana's copy of the leader's input (the leader of nobody here).
fn ftco_800b0af4(_cx: &Cx, fp: &mut Fighter) -> R {
    if fp.kind == Ft_Kind_Popo {
        return Err("ftCo_800B0AF4 (Ice Climbers' follower) not ported".into());
    }
    Ok(())
}

/// `ftCo_800B2AFC`: choose the sensing by CPU kind.
fn ftco_800b2afc(cx: &mut Cx, me: *mut Fighter) -> R {
    let kind = unsafe { (*me).cpu.kind };
    match kind {
        4 | 11 | 12 => ftco_800b24b8(cx, me),
        k => Err(format!("ftCo_800B2AFC: cpu kind {k} not ported")),
    }
}

/// `ftCo_CpuClearTargetModes`.
fn clear_target_modes(c: &mut CpuFighter) {
    c.set_xF9_b2(0);
    c.set_xF9_b4(0);
    c.set_xF9_b3(0);
    c.set_xF9_b5(0);
    c.set_xF9_b6(0);
    c.set_xF9_b7(0);
}

/// `ftCo_CpuShouldAct` / `ftCo_CpuDataShouldAct`.
fn should_act(c: &mut CpuFighter) -> bool {
    if c.x18 != c.x20 && c.x18 != c.x1C {
        c.x60 = 0;
    }
    if c.x18 == 4 {
        false
    } else {
        c.set_xFA_b2(0);
        true
    }
}

/// `ftCo_800B24B8`: sense and act (kind 4: the plain fighting CPU).
fn ftco_800b24b8(cx: &mut Cx, me: *mut Fighter) -> R {
    let fp = unsafe { &mut *me };
    let mut cmd_pos = WVec3 { x: 0.0, y: 0.0, z: 0.0 };
    let cmd = ftco_800a229c(cx, fp)?;
    if cmd != 0 {
        let _ = &mut cmd_pos;
        return Err(format!("ftCo_800AE7AC (recover to camera, cmd {cmd}) not ported"));
    }
    let target = ftco_800a5ce0(cx, me)?;
    fp.cpu.x44 = target;
    if !target.is_null() {
        // ftCo_CpuSetTargetFlags
        fp.cpu.set_xF8_b0(1);
        clear_target_modes(&mut fp.cpu);
        fp.cpu.set_xF9_b1(1);
        // ftCo_CpuTargetAct
        ftco_800a20a0(cx, fp);
        if should_act(&mut fp.cpu) {
            ftco_800a80e4(cx, fp)?;
        }
        ftco_800ade48(cx, me)?;
        return Ok(());
    }
    // ftCo_CpuInitNoTarget(fp, true)
    {
        let c = &mut fp.cpu;
        c.set_xF8_b0(0);
        c.set_xF9_b2(1);
        c.set_xF9_b4(1);
        c.set_xF9_b3(1);
        c.set_xF9_b5(1);
        c.set_xF9_b6(1);
        c.set_xF9_b7(1);
        c.set_xF9_b1(0);
    }
    fp.cpu.x44 = ftco_800a4bec(cx, me)?;
    // ftCo_CpuUpdateCommonItemTarget / SpecialItemTarget
    if !fp.item_gobj.is_null() {
        return Err("ftCo_800A5908 (held item) not ported".into());
    }
    fp.cpu.x4C = if fp.x2168 != 0 { core::ptr::null_mut() } else { ftco_800a5f4c(cx, fp)? };
    fp.cpu.x50 = ftco_800a648c(cx, fp)?;
    // ftCo_CpuActOnNoTarget
    if should_act(&mut fp.cpu) {
        if !fp.cpu.x4C.is_null() {
            return Err("ftCo_800A866C (go for an item) not ported".into());
        }
        let mut at = ftco_800a53dc(cx, me)?;
        if at.is_null() {
            at = fp.cpu.x44;
        }
        ftco_800a75dc(cx, me, at)?;
    }
    ftco_800ade48(cx, me)?;
    Ok(())
}

/// `ftCo_800A5F4C(fp, It_Kind_L_Gun_Ray)`: the common item to go for. Only grabbable items count.
fn ftco_800a5f4c(cx: &Cx, _fp: &Fighter) -> R<*mut Item> {
    for it in cx.items {
        if item_grabbable(it)? {
            return Err("ftCo_800A5F4C (a grabbable item exists) not ported".into());
        }
    }
    Ok(core::ptr::null_mut())
}

/// `Item_IsGrabbable` for the simulated items.
fn item_grabbable(it: &SimItem) -> R<bool> {
    use super::fox::ItemClass as C;
    match it.class {
        C::GreensBlock | C::FoxLaser { .. } | C::FoxGun { .. } | C::SeakHeld { .. } | C::SeakThrown { .. } => Ok(false),
    }
}

/// `ftCo_800A648C`: the nearest hostile (enemy item) of the special kinds.
fn ftco_800a648c(cx: &Cx, _fp: &Fighter) -> R<u32> {
    for it in cx.items {
        let k = it.kind;
        let special = (It_Kind_Kuriboh as i32..It_Kind_Octarock_Stone as i32).contains(&k) || k == It_Kind_Nokonoko as i32 || (k - It_Kind_Patapata as i32) as u32 <= 1 || k == It_Kind_Whitebea as i32;
        if special {
            return Err("ftCo_800A648C (a special item exists) not ported".into());
        }
    }
    Ok(0)
}

/// `gm_8016C6C0` (is_big_loser) / `gm_8016C75C` (KOs made): the match standings. Nobody has lost
/// a stock yet while the stocks are full; later states need `gm_80166378`.
fn match_standing(cx: &Cx, fp: &Fighter) -> R<(i32, i32)> {
    let _ = cx;
    if melee_fighter::stale::stocks(fp.player_id as i32) != 3 {
        return Err("gm_80166378 (match standings after a KO) not ported".into());
    }
    Ok((0, 0))
}

/// `ftCo_IsCpuControlled`.
fn is_cpu(fp: &Fighter) -> bool {
    melee_fighter::stale::slot_type(fp.player_id as i32) == Gm_PKind_Cpu && fp.cpu.kind != 5
}

/// `ftCo_800A53DC`: the fighter to attack (lowest standing, then nearest).
fn ftco_800a53dc(cx: &Cx, me: *mut Fighter) -> R<*mut Fighter> {
    let fp = unsafe { &*me };
    let pick = |skip_cpu: bool| -> R<*mut Fighter> {
        let mut best: *mut Fighter = core::ptr::null_mut();
        let (mut rank, mut dist) = (0i32, 0.0f32);
        for &cur in &cx.fps {
            if cur == me {
                continue;
            }
            let o = unsafe { &*cur };
            if cx.out_of_bounds(&fp.cpu, o.cur_pos.x, o.cur_pos.y) || ftco_is_ally(fp, o) || ftco_800a1c44(o) || inline_d1(o)? {
                continue;
            }
            if skip_cpu && is_cpu(o) {
                continue;
            }
            let (r, _) = match_standing(cx, o)?;
            let d = ftco_800a1ab4(fp, o);
            if best.is_null() {
                best = cur;
                rank = r;
                dist = d;
            } else if rank > r {
                dist = d;
                rank = r;
                best = cur;
            } else if rank == r && dist > d {
                dist = d;
                best = cur;
            }
        }
        Ok(best)
    };
    if b(fp.cpu.xFA_b34()) {
        let t = pick(true)?;
        if !t.is_null() {
            return Ok(t);
        }
    }
    pick(false)
}

/// The floor islands in list order (`mpIsland_80458E88.next` ...).
fn islands(cx: &Cx) -> std::vec::Vec<i32> {
    let mut v = std::vec::Vec::new();
    let mut cur = cx.ground.islands.heads.next;
    while cur != -1 {
        v.push(cur);
        cur = cx.ground.islands.segs[cur as usize].next;
    }
    v
}

/// `ftCo_800A2718`: the island holds something the CPU wants to avoid (a live bomb, a shell).
fn ftco_800a2718(cx: &Cx, island: i32) -> R<bool> {
    if island == -1 {
        return Ok(false);
    }
    for it in cx.items {
        let k = it.kind;
        if k == It_Kind_BombHei as i32 || k == It_Kind_G_Shell as i32 || k == It_Kind_R_Shell as i32 {
            return Err("ftCo_800A2718 (a bomb / shell item) not ported".into());
        }
    }
    match cx.grkind {
        // mpIsland_8005AC8C: the island's collision joint is flagged (Randall's moving platform).
        g if g == Gr_Kind_Story as i32 => Ok(cx.ground.mp_island_8005ac8c(island)),
        g if g == Gr_Kind_Zebes as i32 || g == Gr_Kind_Onett as i32 => Err(format!("ftCo_800A2718 on stage {g} not ported")),
        _ => Ok(false),
    }
}

/// `ftCo_800A4768`: the nearest island end to flee to on the left.
fn ftco_800a4768(cx: &Cx, fp: &Fighter) -> (f32, f32) {
    let mut best = -1.0f32;
    let mut out = (0.0f32, 0.0f32);
    for isl in islands(cx) {
        let s = cx.ground.islands.segs[isl as usize];
        let mut pt = s.x14;
        if !cx.out_of_bounds(&fp.cpu, pt.x, pt.y) {
            let dx = fp.cur_pos.x - pt.x;
            if dx > 0.0 {
                let dy = fp.cur_pos.y - pt.y;
                // FMA: fma(dx, dx, dy * dy)
                let dist = dx.mul_add(dx, dy * dy);
                if best < 0.0 || best > dist {
                    best = dist;
                    out = (dsub(pt.x, 5.0), pt.y);
                }
            }
        }
        if !cx.out_of_bounds(&fp.cpu, pt.x, pt.y) {
            pt = s.x8;
            let dx = fp.cur_pos.x - pt.x;
            if dx < 0.0 {
                let dy = fp.cur_pos.y - pt.y;
                let dist = dx.mul_add(dx, dy * dy);
                if best < 0.0 || best > dist {
                    best = dist;
                    out = (dadd(5.0, pt.x), pt.y);
                }
            }
        }
    }
    out
}

/// `ftCo_800A6700`: the farthest reachable island end from `from` (None: no candidate).
fn ftco_800a6700(cx: &mut Cx, fp: &Fighter, from: WVec3) -> R<Option<WVec3>> {
    let mut best = -1.0f32;
    let mut out = WVec3 { x: 0.0, y: 0.0, z: 0.0 };
    for isl in islands(cx) {
        if ftco_800a2718(cx, isl)? {
            continue;
        }
        let s = cx.ground.islands.segs[isl as usize];
        let a = s.x8;
        let px = dadd(5.0, a.x);
        if cx.probe(px, dadd(5.0, a.y), px, dsub(a.y, 5.0), 0.0)?.is_some() && !cx.out_of_bounds(&fp.cpu, px, a.y) {
            let ddx = px - from.x;
            let dy = a.y - from.y;
            // FMA: fma(ddx, ddx, dy * dy)
            let dist = ddx.mul_add(ddx, dy * dy);
            if dist > best {
                best = dist;
                out = WVec3 { x: px, y: a.y, z: a.z };
            }
        }
        let bb = s.x14;
        let px = dsub(bb.x, 5.0);
        if cx.probe(px, dadd(5.0, bb.y), px, dsub(bb.y, 5.0), 0.0)?.is_some() && !cx.out_of_bounds(&fp.cpu, px, bb.y) {
            let ddx = px - from.x;
            let dy = bb.y - from.y;
            let dist = ddx.mul_add(ddx, dy * dy);
            if dist > best {
                best = dist;
                out = WVec3 { x: px, y: bb.y, z: bb.z };
            }
        }
    }
    if (best as f64) < 0.0 {
        return Ok(None);
    }
    Ok(Some(out))
}

/// `ftCo_800A80E4`: a grounded CPU near its target: head for the farthest island end.
fn ftco_800a80e4(cx: &mut Cx, fp: &mut Fighter) -> R {
    let other = fp.cpu.x44;
    if other.is_null() || fp.ground_or_air == GA_Air as _ {
        return Ok(());
    }
    let o = unsafe { &*other };
    let dx = fp.cur_pos.x - o.cur_pos.x;
    let dy = fp.cur_pos.y - o.cur_pos.y;
    // FMA: fma(dx, dx, dy * dy)
    if !((sqrtf(dx.mul_add(dx, dy * dy)) as f64) > 50.0) {
        let from = WVec3 { x: o.cur_pos.x, y: o.cur_pos.y, z: o.cur_pos.z };
        if let Some(v) = ftco_800a6700(cx, fp, from)? {
            ftco_800a1f3c(cx, fp, v.x, v.y, 5.0)?;
        }
    }
    Ok(())
}

/// `ftCo_800A20A0`: a grounded target within twice the character's threshold.
fn ftco_800a20a0(cx: &Cx, fp: &mut Fighter) {
    let o = fp.cpu.x44;
    if !o.is_null() && fp.ground_or_air != GA_Air as _ {
        let t = cx.dist_thresh.get(fp.kind as usize).copied().unwrap_or(0.0);
        let d = ftco_800a1ab4(fp, unsafe { &*o });
        fp.cpu.set_xF8_b6((d < 2.0f32 * t) as u8);
    } else {
        fp.cpu.set_xF8_b6(0);
    }
}

/// The `ftCo_800A1CC4` stage-island hook after a destination change: only Great Bay / Shrine.
fn island_hook(cx: &Cx) -> R {
    if cx.grkind == Gr_Kind_GreatBay as i32 || cx.grkind == Gr_Kind_Shrine as i32 {
        return Err("ftCo_800A1CC4 (stage island list) not ported".into());
    }
    Ok(())
}

/// `ftCo_800A75DC(fp, target)`: head for the target (NULL: nothing).
fn ftco_800a75dc(cx: &mut Cx, me: *mut Fighter, target: *mut Fighter) -> R {
    if target.is_null() {
        return Ok(());
    }
    let fp = unsafe { &mut *me };
    let t = unsafe { &*target };
    let tol = fp.cpu.x56C + t.cpu.x564;
    if t.ground_or_air == GA_Air as _ {
        // The floor under the target (hazard lines dropped).
        let hit = cx.probe(t.cur_pos.x, dadd(10.0, t.cur_pos.y), t.cur_pos.x, t.cur_pos.y - 1000.0f32, 0.0)?;
        if let Some(h) = hit {
            let isl = cx.ground.mp_island_8005ab54(h.line_id);
            if !ftco_800a2718(cx, isl)? {
                ftco_800a1f3c(cx, fp, h.pos.x, h.pos.y, tol)?;
                if isl != -1 {
                    let s = cx.ground.islands.segs[isl as usize];
                    let d = (s.x14.x - fp.cpu.x54.x).abs();
                    if (d as f64) < 5.0 {
                        if fp.cpu.x60 == 0 {
                            fp.cpu.x54.x = dsub(s.x14.x, 5.0);
                            fp.cpu.x54.y = s.x14.y;
                            fp.cpu.x38 = tol;
                            island_hook(cx)?;
                        }
                    } else {
                        let d = (s.x8.x - fp.cpu.x54.x).abs();
                        if (d as f64) < 5.0 {
                            ftco_800a1f3c(cx, fp, dadd(5.0, s.x8.x), s.x8.y, tol)?;
                        }
                    }
                }
            }
        } else {
            let (x, y) = ftco_800a4768(cx, fp);
            ftco_800a1f3c(cx, fp, x, y, tol)?;
        }
    } else {
        let isl_t = cx.ground.mp_island_8005ab54(t.coll_data.floor.index);
        if !ftco_800a2718(cx, isl_t)? {
            ftco_800a1f3c(cx, fp, t.cur_pos.x, t.cur_pos.y, tol)?;
            let same = if fp.ground_or_air == GA_Air as _ || t.ground_or_air == GA_Air as _ {
                false
            } else {
                let i0 = cx.ground.mp_island_8005ab54(fp.coll_data.floor.index);
                i0 != -1 && isl_t == i0
            };
            if !same && isl_t != -1 && (fp.cpu.x54.y - fp.cur_pos.y) as f64 > 0.0 {
                let s = cx.ground.islands.segs[isl_t as usize];
                if (fp.cpu.x54.x - fp.cur_pos.x) as f64 > 0.0 {
                    if fp.cur_pos.x < s.x8.x && fp.cpu.x60 == 0 {
                        fp.cpu.x54.x = dadd(5.0, s.x8.x);
                        fp.cpu.x54.y = s.x8.y;
                        fp.cpu.x38 = tol;
                        island_hook(cx)?;
                    }
                } else if fp.cur_pos.x > s.x14.x {
                    ftco_800a1f3c(cx, fp, dsub(s.x14.x, 5.0), s.x14.y, tol)?;
                }
            }
        }
    }
    Ok(())
}

/// `ftCo_800ADE48`: recompute the destination and switch behaviour on the fighter's state.
fn ftco_800ade48(cx: &mut Cx, me: *mut Fighter) -> R {
    let fp = unsafe { &mut *me };
    // The destination: the floor under (x54, y54 +- 5), else under the fighter.
    let (x, y) = (fp.cpu.x54.x, fp.cpu.x54.y);
    let mut found = cx.probe(x, dadd(5.0, y), x, dsub(y, 5.0), 0.0)?;
    let mut done = false;
    if found.is_some() {
        // in_bounds is the C's inverted name: 0 when the point is inside the zone -> keep.
        let oob_in = cx.out_of_bounds(&fp.cpu, fp.cpu.x54.x, fp.cpu.x54.y);
        if !oob_in {
            done = true;
        }
    }
    if !done {
        fp.cpu.set_xFA_b2(0);
        if fp.ground_or_air == GA_Air as _ {
            let (x, y) = (fp.cur_pos.x, fp.cur_pos.y);
            found = cx.probe(x, y, x, y - 1000.0f32, 0.0)?;
            match found {
                Some(h) => {
                    fp.cpu.x54.x = h.pos.x;
                    fp.cpu.x54.y = h.pos.y;
                }
                None => {
                    fp.cpu.x54.x = fp.cur_pos.x;
                    fp.cpu.x54.y = fp.cur_pos.y;
                }
            }
        } else {
            fp.cpu.x54.x = fp.cur_pos.x;
            fp.cpu.x54.y = fp.cur_pos.y;
        }
    }
    let dy = fp.cur_pos.y - fp.cpu.x54.y;
    let dx = fp.cur_pos.x - fp.cpu.x54.x;
    // FMA: fma(dx, dx, dy * dy)
    fp.cpu.x5C = sqrtf(dx.mul_add(dx, dy * dy));
    if fp.cpu.x88 > 0 {
        fp.cpu.x88 -= 1;
    }
    // gm_8016C75C(gobj): the match-end rank of the fighter; with no KOs yet it is 0.
    let rank = 0;
    if fp.cpu.x8C != rank {
        fp.cpu.x88 = 0x12C;
        fp.cpu.x8C = rank;
    }
    if fp.cpu.kind == 0xE {
        if fp.cpu.x7C % 120 == 0 {
            let r = cx.rand.randf();
            fp.cpu.set_xF8_b5((r as f64 > 0.5) as u8);
        }
    } else if fp.cpu.x7C % 400 == 0 {
        let r = cx.rand.randf();
        fp.cpu.set_xF8_b5((r as f64 > 0.5) as u8);
    }
    if !b(fp.anon_221b.x221B_b5()) {
        fp.cpu.x94 = 0;
    }
    // Behaviour switches, in the C's order. Each `return true` ends the function.
    if fp.cpu.x18 == 0x12 {
        return Ok(());
    }
    let mut switch_cmd = 0;
    if fp.kind == Ft_Kind_GKoops {
    } else if fp.cpu.kind == 0xF || fp.cpu.kind == 0 {
    } else if !b(fp.x221A_b3()) {
    } else {
        // Just hit: xF9_b0 off, xFA_b1 = (0.1 * level > HSD_Randf()). `switch_cmd` is not assigned on this path in
        // the C (an uninitialised read); MELEE_CPU_SWITCH selects what the build leaves in it.
        fp.cpu.set_xF9_b0(0);
        let r = cx.rand.randf();
        fp.cpu.set_xFA_b1(((0.1f32 * fp.cpu.level as f32) > r) as u8);
        if std::env::var_os("MELEE_CPU_SWITCH").is_some() {
            switch_cmd = 1;
        }
    }
    if switch_cmd != 0 {
        return Err("ftCo_800ADE48: the uninitialised switch_cmd path (ftCo_800B4A78) is not ported".into());
    }
    let _ = &mut switch_cmd;
    let cur_cmd = fp.cpu.x18;
    let motion_id = fp.motion_id;
    if cur_cmd == 0x11 {
        return Ok(());
    }
    // inlineH0: BarrelWait / Barrel.
    if motion_id == ftCo_MS_BarrelWait as i32 || motion_id == ftCo_MS_Barrel as i32 {
        return Err("ftCo_800ADE48: barrel not ported".into());
    }
    if cur_cmd == 0x13 {
        return Ok(());
    }
    if motion_id == 0x131 || motion_id == 0x132 {
        cs::ftco_800b4a78(fp);
        fp.cpu.x18 = 0x13;
        return Ok(());
    }
    if cur_cmd == 5 {
        return Ok(());
    }
    if matches!(motion_id, 0xBF | 0xB7 | 0xC0 | 0xB8) {
        cs::ftco_800b4a78(fp);
        fp.cpu.x18 = 5;
        return Ok(());
    }
    if cur_cmd == 6 {
        return Ok(());
    }
    if matches!(motion_id, 0xFC | 0xFD) {
        cs::ftco_800b4a78(fp);
        fp.cpu.x18 = 6;
        return Ok(());
    }
    if cur_cmd == 9 {
        return Ok(());
    }
    if ftco_is_grabbing(fp) {
        cs::ftco_800b4a78(fp);
        fp.cpu.x18 = 9;
        return Ok(());
    }
    if fp.cpu.x18 == 4 {
        return Ok(());
    }
    if ftco_800a2c80(cx, fp)? {
        cs::ftco_800b4a78(fp);
        fp.cpu.x18 = 4;
        return Ok(());
    }
    let cur_cmd = fp.cpu.x18;
    if cur_cmd == 0xF {
        return Ok(());
    }
    if fp.cpu.level >= 5 && fp.motion_id == 0x26 {
        cs::ftco_800b4a78(fp);
        fp.cpu.x18 = 0xF;
        return Ok(());
    }
    if cur_cmd == 0x10 {
        return Ok(());
    }
    let m = fp.motion_id;
    if m == 0xE0 || m == 0xE3 || (0x10A..=0x10E).contains(&m) {
        cs::ftco_800b4a78(fp);
        fp.cpu.x18 = 0x10;
        return Ok(());
    }
    if cur_cmd == 7 {
        ftco_800bb9b4(cx, me)?;
        return Ok(());
    }
    if ftco_800a5acc(fp) {
        ftco_800bb9b4(cx, me)?;
        if fp.cpu.xF8_b12() != 0 {
            cs::ftco_800b4a78(fp);
            fp.cpu.x18 = 7;
            return Ok(());
        }
    }
    if fp.cpu.x18 == 3 {
        return Ok(());
    }
    if ftco_800b9cbc(cx, fp)? {
        cs::ftco_800b4a78(fp);
        fp.cpu.x18 = 3;
        return Ok(());
    }
    if fp.cpu.x18 == 2 {
        return Ok(());
    }
    if ftco_800b8a9c(cx, fp)? {
        cs::ftco_800b4a78(fp);
        fp.cpu.x18 = 2;
        return Ok(());
    }
    if fp.cpu.x18 == 8 {
        return Ok(());
    }
    let r = ftco_800b732c(cx, fp)?;
    fp.cpu.set_xF8_b34(r as u8);
    if r != 0 {
        fp.cpu.x18 = 8;
        return Ok(());
    }
    if fp.cpu.x18 == 0xD {
        return Ok(());
    }
    if ftco_800a3710(fp)? {
        fp.cpu.x18 = 0xD;
        return Ok(());
    }
    if fp.cpu.x18 == 0xE {
        return Ok(());
    }
    // The item-use tail (x18 != 0xE): holding an item switches to 0xE.
    if !fp.item_gobj.is_null() {
        return Err("ftCo_800ADE48: held item not ported".into());
    }
    if fp.cpu.x18 == 0xA {
        return Ok(());
    }
    if fp.cpu.x20 != 0 && ftco_800a3554(cx, fp, 0.0)? {
        fp.cpu.x18 = fp.cpu.x20;
    }
    Ok(())
}

/// `ftCo_IsGrabbing`.
fn ftco_is_grabbing(fp: &Fighter) -> bool {
    if fp.motion_id == ftCo_MS_CatchWait as i32 {
        return true;
    }
    let k = fp.kind;
    if k == Ft_Kind_Donkey && fp.motion_id >= ftDk_MS_ThrowFWait0 as i32 && fp.motion_id <= ftDk_MS_ThrowFKneebend as i32 {
        return true;
    }
    if (k == Ft_Kind_Koopa || k == Ft_Kind_GKoops) && fp.motion_id == ftKp_MS_SpecialSHit0_1 as i32 {
        return true;
    }
    false
}

/// `ftCo_800A2C80`: about to fall into the blast zone (and no way to stop it).
fn ftco_800a2c80(cx: &mut Cx, fp: &Fighter) -> R<bool> {
    if b(fp.cpu.xFA_b6()) {
        return Ok(false);
    }
    if fp.ground_or_air == GA_Ground as _ {
        // ftCo_800A2C80_inline0: the floor is a hazard line.
        return cx.ignored_floor(fp.coll_data.floor.index);
    }
    if fp.motion_id == 0xF4 {
        return Ok(false);
    }
    let (mut v, vy, vz) = (fp.pos_delta.x, fp.pos_delta.y, fp.pos_delta.z);
    let mag = v * v + vy * vy;
    if (mag as f64) < 0.00001 && (mag as f64) > -0.00001 {
        return Ok(false);
    }
    if vy as f64 > 0.0 {
        return Ok(false);
    }
    if v < 0.0 {
        v = -v;
    }
    if (cs::ext::lb_8000D008(vy, v) as f64) > -1.0471975430846214 {
        return Ok(false);
    }
    if b(fp.cpu.xFA_b5()) {
        return Ok(false);
    }
    if cx.grkind == Gr_Kind_Inishie1 as i32 || cx.grkind == Gr_Kind_Fourside as i32 {
        return Err("ftCo_800A2C80: Pokemon/Fourside branch not ported".into());
    }
    // lbVector_Normalize(&dir): FMA fma(z, z, fma(x, x, y * y)).
    let (mut dx, mut dy, mut dz) = (fp.pos_delta.x, vy, vz);
    let len = sqrtf(dz.mul_add(dz, dx.mul_add(dx, dy * dy)));
    if len != 0.0 {
        let inv = 1.0f32 / len;
        dx *= inv;
        dy *= inv;
        dz *= inv;
    }
    let _ = dz;
    let ax = fp.coll_data.cur_pos.x + fp.coll_data.ecb.bottom.x;
    let ay = fp.coll_data.cur_pos.y + fp.coll_data.ecb.bottom.y;
    // FMA: 1000.0f * dir + a
    let ex = 1000.0f32.mul_add(dx, ax);
    let ey = 1000.0f32.mul_add(dy, ay);
    let inb = |cx: &Cx, c: &CpuFighter, p: WVec3| !cx.out_of_bounds(c, p.x, p.y);
    if let Some(h) = cx.probe(ax, ay, ex, ey, 0.0)? {
        if inb(cx, &fp.cpu, WVec3 { x: h.pos.x, y: h.pos.y, z: 0.0 }) {
            return Ok(false);
        }
    }
    if let Some(h) = cx.ground.mp_check_left_wall(ax, ay, ex, ey, -1, -1) {
        if inb(cx, &fp.cpu, WVec3 { x: h.pos.x, y: h.pos.y, z: 0.0 }) {
            return Ok(false);
        }
    }
    if let Some(h) = cx.ground.mp_check_right_wall(ax, ay, ex, ey, -1, -1) {
        if inb(cx, &fp.cpu, WVec3 { x: h.pos.x, y: h.pos.y, z: 0.0 }) {
            return Ok(false);
        }
    }
    Ok(true)
}




/// `ftCo_800A3710`: go pick up the targeted item.
fn ftco_800a3710(fp: &mut Fighter) -> R<bool> {
    if !b(fp.cpu.xF9_b7()) || fp.cpu.x4C.is_null() {
        return Ok(false);
    }
    Err("ftCo_800A3710 (an item target) not ported".into())
}

/// `ftCo_800A21FC`: the destination lies on the island the fighter stands on.
fn ftco_800a21fc(cx: &mut Cx, fp: &Fighter) -> bool {
    if fp.ground_or_air == GA_Air as _ {
        return false;
    }
    let isl = cx.ground.mp_island_8005ab54(fp.coll_data.floor.index);
    if isl == -1 {
        return false;
    }
    let v = WVec3 { x: fp.cpu.x54.x, y: dadd(5.0, fp.cpu.x54.y), z: 0.0 };
    cx.ground.mp_island_8005ac14(v, -10.0) == isl
}

/// `ftCo_800A3554`: the destination is reached (within `x38 + arg`).
fn ftco_800a3554(cx: &mut Cx, fp: &mut Fighter, arg: f32) -> R<bool> {
    if fp.ground_or_air == GA_Air as _ {
        return Ok(false);
    }
    if !ftco_800a21fc(cx, fp) {
        return Ok(false);
    }
    let x = fp.cpu.x54.x - fp.cur_pos.x;
    let y = fp.cpu.x54.y - fp.cur_pos.y;
    // FMA: fma(x, x, y * y)
    if sqrtf(x.mul_add(x, y * y)) < fp.cpu.x38 + arg {
        if fp.cpu.x60 != 0 {
            fp.cpu.x60 = 0;
            fp.cpu.x54 = fp.cpu.x64;
            if cx.grkind == Gr_Kind_GreatBay as i32 || cx.grkind == Gr_Kind_Shrine as i32 {
                return Err("ftCo_800A1CC4 (stage island list) not ported".into());
            }
            return Ok(false);
        }
        return Ok(true);
    }
    Ok(false)
}


// ---- ftcpuattack.c ----

/// `ftCo_AttackEntry` (PlCo, 0x24 bytes; the list ends at `cmd == 0`).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct AttackEntry {
    pub cmd: i32,
    pub x04: i32,
    pub x08: f32,
    pub x0c: f32,
    pub x10: f32,
    pub x14: f32,
    pub weight: f32,
    pub x1c: i32,
    pub x20: i32,
}

/// `Fighter_804D64FC` field offsets (the per-character tables of attack lists).
pub(super) const T_CMDSCRIPTS: u32 = 0x00;
pub(super) const T_GROUND: u32 = 0x04; // x4
pub(super) const T_AIR: u32 = 0x08; // x8
pub(super) const T_RANGED: u32 = 0x0C; // xC
pub(super) const T_SMASH: u32 = 0x10; // x10
pub(super) const T_SPECIAL: u32 = 0x14; // x14
pub(super) const T_WEAPON: u32 = 0x18; // x18
pub(super) const T_EDGE: u32 = 0x1C; // x1C

impl Cx<'_> {
    fn be(&self, at: u32) -> u32 {
        u32::from_be_bytes(self.plco[at as usize..at as usize + 4].try_into().unwrap())
    }

    /// `Fighter_804D64FC-><field>[kind]`: the attack list, `None` for a NULL pointer.
    fn attack_list(&self, field: u32, kind: FighterKind) -> Option<std::vec::Vec<AttackEntry>> {
        let arr = self.be(self.tbl + field);
        let p = self.be(arr + 4 * kind as u32);
        if p == 0 {
            return None;
        }
        let mut v = std::vec::Vec::new();
        let mut at = p;
        loop {
            let cmd = self.be(at) as i32;
            if cmd == 0 {
                break;
            }
            let f = |o: u32| f32::from_bits(self.be(at + o));
            v.push(AttackEntry { cmd, x04: self.be(at + 4) as i32, x08: f(8), x0c: f(0xC), x10: f(0x10), x14: f(0x14), weight: f(0x18), x1c: self.be(at + 0x1C) as i32, x20: self.be(at + 0x20) as i32 });
            at += 0x24;
        }
        Some(v)
    }

    /// `ftCo_800B4880(fp, idx)`: append command script `idx`.
    fn run_cmdscript(&self, fp: &mut Fighter, idx: i32) {
        let arr = self.be(self.tbl + T_CMDSCRIPTS);
        let mut at = self.be(arr + 4 * idx as u32) as usize;
        loop {
            let c = self.plco[at];
            cs::ftco_800b463c(fp, c);
            if c as u32 == CpuCmd_Done {
                break;
            }
            if c as u32 > CpuCmd_OneArgEnd {
                at += 1;
                cs::ftco_800b463c(fp, self.plco[at]);
            }
            if c as u32 > CpuCmd_ZeroArgEnd {
                // (the C re-tests the same byte; a two-arg command has a second argument)
                at += 1;
                cs::ftco_800b463c(fp, self.plco[at]);
            }
            at += 1;
        }
    }
}

/// Push onto the 8-entry avoid list (`xCC_array`, count `xEC`).
fn push_avoid(c: &mut CpuFighter, v: i32) -> bool {
    if (c.xEC as u32) < 8 {
        c.xCC_array[c.xEC as usize] = v;
        c.xEC += 1;
        true
    } else {
        false
    }
}

/// `ftCo_800B6208`: a weighted pick of an attack command from a ranged list.
fn ftco_800b6208(cx: &mut Cx, arr: &[AttackEntry]) -> R<i32> {
    let r = cx.rand.randf();
    let mut sum = 0.0f32;
    for e in arr {
        sum += e.weight;
    }
    if arr.is_empty() {
        return Ok(0);
    }
    if sum < 0.00001 && sum > -0.00001 {
        return Ok(0);
    }
    let inv = (1.0f64 / sum as f64) as f32;
    let mut acc = 0.0f32;
    for e in arr {
        acc += e.weight;
        if acc * inv >= r {
            return Ok(e.cmd);
        }
    }
    Err("ftCo_800B6208: HSD_ASSERT(0x28A)".into())
}

/// `ftCo_800B630C`: the fighter is in an attacking motion.
fn ftco_800b630c(fp: &Fighter) -> R<bool> {
    let m = fp.motion_id;
    if m >= ftCo_MS_Attack11 as i32 && m <= ftCo_MS_AttackAirLw as i32 {
        return Ok(true);
    }
    // ftCo_800A5A90: item swings / shots.
    if (m >= ftCo_MS_SwordSwing1 as i32 && m <= ftCo_MS_StarRodSwingDash as i32) || (m >= ftCo_MS_LGunShoot as i32 && m <= ftCo_MS_ItemScopeAirEndEmpty as i32) {
        return Ok(true);
    }
    let k = fp.kind;
    if k == Ft_Kind_Donkey || k == Ft_Kind_Kirby || k == Ft_Kind_Peach {
        return Err("ftCo_800B630C (Donkey Kong / Kirby / Peach) not ported".into());
    }
    if m < ftCo_MS_Count as i32 {
        return Ok(false);
    }
    Ok(true)
}

/// `ftCo_800B63D8`: wait a level-scaled random time.
fn ftco_800b63d8(cx: &mut Cx, fp: &mut Fighter) {
    let r = cx.rand.randf();
    let (n, add) = match fp.cpu.level {
        0 => (40, 5),
        1 => (35, 5),
        2 => (30, 5),
        3 => (25, 5),
        4 => (25, 0),
        5 => (20, 0),
        6 => (15, 0),
        7 => (10, 0),
        8 => (5, 0),
        _ => return,
    };
    cmd1(fp, CpuCmd_WaitFor, (n as f32 * r) as i32 + add);
}

/// `ftCo_800B9F90`: tap shield (R), holding for a level-scaled time while already shielding.
fn ftco_800b9f90(cx: &mut Cx, fp: &mut Fighter) {
    if fp.motion_id == ftCo_MS_Guard as i32 {
        let t = 9 - fp.cpu.level;
        neutral_stick(fp);
        cmd0(fp, CpuCmd_PressR);
        if t != 0 {
            let r = cx.rand.randf();
            cmd1(fp, CpuCmd_WaitFor, t * ((4.0f32 * r) as i32 + 4));
        }
        done(fp);
    } else {
        neutral_stick(fp);
        cmd0(fp, CpuCmd_PressR);
        done(fp);
    }
}

/// `ftCo_800BA160` / `ftCo_800BA224`: roll right / left.
fn ftco_800ba160(fp: &mut Fighter) {
    neutral_stick(fp);
    cmd0(fp, CpuCmd_ReleaseR);
    cmd1(fp, CpuCmd_WaitFor, 1);
    cmd1(fp, CpuCmd_SetLstickX, 0x7F);
    cmd0(fp, CpuCmd_PressR);
    cmd1(fp, CpuCmd_WaitFor, 2);
    cmd0(fp, CpuCmd_ReleaseR);
    cmd1(fp, CpuCmd_WaitFor, 5);
    cmd1(fp, CpuCmd_SetLstickX, 0);
    done(fp);
}
fn ftco_800ba224(fp: &mut Fighter) {
    neutral_stick(fp);
    cmd0(fp, CpuCmd_ReleaseR);
    cmd1(fp, CpuCmd_WaitFor, 1);
    cmd1(fp, CpuCmd_SetLstickX, -0x7F);
    cmd0(fp, CpuCmd_PressR);
    cmd1(fp, CpuCmd_WaitFor, 2);
    cmd0(fp, CpuCmd_ReleaseR);
    cmd1(fp, CpuCmd_WaitFor, 5);
    cmd1(fp, CpuCmd_SetLstickX, 0);
    done(fp);
}

/// `ftCo_800A2A70`: distance to the island's end (left when `left`), -1 when airborne / no island.
fn ftco_800a2a70(cx: &Cx, fp: &Fighter, left: bool) -> f32 {
    let isl = island_of(cx, fp);
    if isl == -1 {
        return -1.0;
    }
    let s = cx.ground.islands.segs[isl as usize];
    let e = if left { s.x8 } else { s.x14 };
    let x = e.x - fp.cur_pos.x;
    let y = e.y - fp.cur_pos.y;
    // FMA: fma(x, x, y * y)
    sqrtf(x.mul_add(x, y * y))
}

/// `ftCo_800BA2E8`: roll away from the nearer end when the island end is far.
fn ftco_800ba2e8(cx: &Cx, fp: &mut Fighter, target: &Fighter) -> bool {
    let t = cx.dist_thresh.get(fp.kind as usize).copied().unwrap_or(0.0);
    if (fp.cur_pos.x - target.cur_pos.x) as f64 > 0.0 {
        if ftco_800a2a70(cx, fp, false) > t {
            ftco_800ba160(fp);
            return true;
        }
        if ftco_800a2a70(cx, fp, true) > t {
            ftco_800ba224(fp);
            return true;
        }
        false
    } else {
        if ftco_800a2a70(cx, fp, true) > t {
            ftco_800ba224(fp);
            return true;
        }
        if ftco_800a2a70(cx, fp, false) > t {
            ftco_800ba160(fp);
            return true;
        }
        false
    }
}

/// `ftCo_800A0098` (the walk-away script).
fn ftco_800a0098(fp: &mut Fighter) {
    cmd1(fp, 0x80, 0);
    cmd1(fp, 0x81, 0);
    cmd1(fp, 0x8E, 1);
    cmd1(fp, 0x91, 0xB0);
    cmd1(fp, 0x8E, 1);
    cmd1(fp, 0x81, 0);
    cmd1(fp, 0x8E, 0xA);
    cmd1(fp, 0x80, 0);
    cmd0(fp, 0x7F);
}

/// `ftCo_800A0C8C`: stop (command 127).
fn ftco_800a0c8c(fp: &mut Fighter) {
    cmd0(fp, 127);
}

/// `ftCo_800A3134`.
fn ftco_800a3134(fp: &Fighter) -> i32 {
    let m = fp.motion_id;
    if m == ftCo_MS_DownBoundD as i32 || m == ftCo_MS_DownBoundU as i32 {
        1
    } else if m == ftCo_MS_DownWaitD as i32 || m == ftCo_MS_DownWaitU as i32 {
        2
    } else {
        0
    }
}

/// `ftCo_800A3200`: the fighter hangs on a ledge.
fn ftco_800a3200(fp: &Fighter) -> i32 {
    let m = fp.motion_id;
    if m == ftCo_MS_CliffCatch as i32 {
        1
    } else if m == ftCo_MS_CliffWait as i32 {
        2
    } else {
        0
    }
}

/// `ftCo_800B4AB0`: the weighted attack choice from a list for the current geometry.
/// Returns the attack's command-script index (0: none) and sets the hit-reach boxes `x6C` / `x74`.
fn ftco_800b4ab0(cx: &mut Cx, fp: &mut Fighter, target: &Fighter, list: Option<std::vec::Vec<AttackEntry>>) -> R<i32> {
    fp.cpu.x74.y = 0.0;
    fp.cpu.x74.x = 0.0;
    fp.cpu.x6C.y = 0.0;
    fp.cpu.x6C.x = 0.0;
    // (f32)(0.5 * x570 + 0.5): FMA f64.
    let half_range = 0.5f64.mul_add(fp.cpu.x570 as f64, 0.5) as f32;
    if std::env::var_os("MELEE_GOLDEN_RNG").is_some() && fp.player_id == 1 {
        eprintln!("A4 pd {:08X} {:08X} tpd {:08X} {:08X} lv {} xec {} x7C {} list {}", fp.pos_delta.x.to_bits(), fp.pos_delta.y.to_bits(), target.pos_delta.x.to_bits(), target.pos_delta.y.to_bits(), fp.cpu.level, fp.cpu.xEC, fp.cpu.x7C, list.is_some() as i32);
    }
    let Some(list) = list else { return Ok(0) };
    let fp_term_neg = -fp.co_attrs.terminal_velocity;
    let tgt_term_neg = -target.co_attrs.terminal_velocity;
    let (fp_x, fp_y, fp_vx, fp_vy, fp_grav) = (fp.cur_pos.x, fp.cur_pos.y, fp.pos_delta.x, fp.pos_delta.y, fp.co_attrs.gravity);
    let (tx, ty, tvx, tvy, tgrav) = (target.cur_pos.x, target.cur_pos.y, target.pos_delta.x, target.pos_delta.y, target.co_attrs.gravity);
    let (range_f, range_b) = if target.facing_dir as f64 > 0.0 { (target.cpu.x55C, target.cpu.x560) } else { (target.cpu.x560, target.cpu.x55C) };
    let x568 = target.cpu.x568;
    let mut sel: std::vec::Vec<AttackEntry> = std::vec::Vec::new();
    let near0 = |g: f32| g < 0.00001 && g > -0.00001;
    for l in &list {
        if l.x20 > fp.cpu.level {
            continue;
        }
        if (0..fp.cpu.xEC as usize).any(|j| fp.cpu.xCC_array[j] == l.cmd) {
            continue;
        }
        let t = l.x04 as f32;
        // FMA: tvx * t + tx, fpVx * t + fpX
        let relx = tvx.mul_add(t, tx) - fp_vx.mul_add(t, fp_x);
        let fp_pred_y = if target.ground_or_air == GA_Air as _ {
            let v = if near0(fp_grav) { 1000.0f32 } else { -(fp_term_neg - fp_y) / fp_grav };
            if v <= 0.0 {
                fp_vy.mul_add(t, fp_y)
            } else if t < v {
                let sq = sqrtf(t);
                (fp_y as f64 + (-0.5f64).mul_add((fp_grav * sq) as f64, (fp_vy * t) as f64)) as f32
            } else {
                let sq = sqrtf(v);
                (fp_y as f64 + (((fp_term_neg * (t - v)) as f64) + (-0.5f64).mul_add((fp_grav * sq) as f64, (fp_vy * t) as f64))) as f32
            }
        } else {
            fp_vy.mul_add(t, fp_y)
        };
        let rel_pred_y = if target.ground_or_air == GA_Air as _ {
            let v = if near0(tgrav) { 1000.0f32 } else { -(tgt_term_neg - tvy) / tgrav };
            if v <= 0.0 {
                tvy.mul_add(t, ty) - fp_pred_y
            } else if t < v {
                let sq = sqrtf(t);
                ((-0.5f64).mul_add((tgrav * sq) as f64, tvy.mul_add(t, ty) as f64) - fp_pred_y as f64) as f32
            } else {
                let sq = sqrtf(v);
                (((tgt_term_neg * (t - v)) as f64 + (-0.5f64).mul_add((tgrav * sq) as f64, tvy.mul_add(t, ty) as f64)) - fp_pred_y as f64) as f32
            }
        } else {
            tvy.mul_add(t, ty) - fp_pred_y
        };
        let sc = fp.x34_scale.y;
        let (mut dirx, mut diry) = if fp.facing_dir as f64 > 0.0 { (l.x08 * sc, l.x0c * sc) } else { (-l.x0c * sc, -l.x08 * sc) };
        dirx *= half_range;
        diry *= half_range;
        let upper = (l.x14 * sc) * half_range;
        let lower = (l.x10 * sc) * half_range;
        if upper > rel_pred_y && lower < rel_pred_y + x568 && dirx < relx + range_f && diry > relx - range_b {
            if fp.cpu.xC8 != 0 {
                for j in 0..fp.cpu.xC8 as usize {
                    if fp.cpu.xA8_array[j] == l.cmd {
                        sel.push(*l);
                    }
                }
            } else if fp.cpu.x80 % l.x1c == 0 {
                sel.push(*l);
            }
        }
    }
    if sel.is_empty() {
        return Ok(0);
    }
    let r = cx.rand.randf();
    let mut sum = 0.0f32;
    for e in &sel {
        sum += e.weight;
    }
    if sum < 0.00001 && sum > -0.00001 {
        return Ok(0);
    }
    let inv = (1.0f64 / sum as f64) as f32;
    let mut acc = 0.0f32;
    for e in &sel {
        acc += e.weight;
        if acc * inv >= r {
            // ftCo_CpuSelectAttack
            let sc = fp.x34_scale.y;
            fp.cpu.x6C.x = sc * e.x08;
            fp.cpu.x6C.y = sc * e.x10;
            fp.cpu.x74.x = sc * e.x0c;
            fp.cpu.x74.y = sc * e.x14;
            return Ok(e.cmd);
        }
    }
    Err("ftCo_800B4AB0: HSD_ASSERT(0xFA)".into())
}

/// `ftCo_CpuCanUseRangedAttack`: the straight line to the target is blocked, or the floor ahead
/// is missing.
fn can_use_ranged(cx: &mut Cx, fp: &Fighter, target: *mut Fighter, range: f32) -> R<bool> {
    if !target.is_null() {
        let t = unsafe { &*target };
        let h = cx.ground.mp_check_all(-1, -1, fp.cur_pos.x, fp.cur_pos.y, t.cur_pos.x, t.cur_pos.y);
        if std::env::var_os("MELEE_GOLDEN_RNG").is_some() {
            if let Some(hh) = &h { let j = cx.ground.mp_joint_from_line(hh.line_id); eprintln!("RNGJ line {} joint {} jflags {:08X} lflags {:08X}", hh.line_id, j, cx.ground.joints[j as usize].flags, cx.ground.lines[hh.line_id as usize].flags); }
            eprintln!("RNGA p{} pos {:08X} {:08X} tgt {:08X} {:08X} -> {:?}", fp.player_id, fp.cur_pos.x.to_bits(), fp.cur_pos.y.to_bits(), t.cur_pos.x.to_bits(), t.cur_pos.y.to_bits(), h.as_ref().map(|h| (h.line_id, h.pos.x.to_bits(), h.pos.y.to_bits())));
        }
        if h.is_some() {
            return Ok(true);
        }
    }
    let x = if fp.facing_dir as f64 > 0.0 { fp.cur_pos.x + range } else { fp.cur_pos.x - range };
    let y = fp.cur_pos.y;
    let r = cx.probe(x, dadd(5.0, y), x, dsub(y, 1000.0), 0.0)?;
    if std::env::var_os("MELEE_GOLDEN_RNG").is_some() {
        eprintln!("RNG p{} pos {:08X} {:08X} fac {:08X} range {} -> {:?}", fp.player_id, fp.cur_pos.x.to_bits(), fp.cur_pos.y.to_bits(), fp.facing_dir.to_bits(), range, r.as_ref().map(|h| (h.line_id, h.pos.x.to_bits(), h.pos.y.to_bits())));
    }
    Ok(r.is_none())
}

/// `ftCo_800B77E8`: fill the avoid list (`xCC`) with moves not to use right now.
fn ftco_800b77e8(cx: &mut Cx, fp: &mut Fighter) -> R {
    if fp.cpu.kind == 7 || ftco_800a1c44(fp) {
        push_avoid(&mut fp.cpu, 0x34);
    }
    let kind = fp.kind;
    let target = fp.cpu.x44;
    if kind == Ft_Kind_Mario {
        if can_use_ranged(cx, fp, target, 22.0)? {
            push_avoid(&mut fp.cpu, 0x1F);
        }
    } else if kind == Ft_Kind_Fox || kind == Ft_Kind_Falco {
        if can_use_ranged(cx, fp, target, 62.0)? {
            push_avoid(&mut fp.cpu, 0x1B);
        }
    } else if kind == Ft_Kind_Captain || kind == Ft_Kind_Ganon {
        if can_use_ranged(cx, fp, target, 102.0)? {
            push_avoid(&mut fp.cpu, 0x26);
        }
        if can_use_ranged(cx, fp, target, 58.0)? {
            let ok = push_avoid(&mut fp.cpu, 0x1B);
            if ok {
                push_avoid(&mut fp.cpu, 0x37);
                push_avoid(&mut fp.cpu, 0x38);
            }
        }
    } else if kind == Ft_Kind_Pikachu || kind == Ft_Kind_Pichu {
        if can_use_ranged(cx, fp, target, 57.0)? {
            push_avoid(&mut fp.cpu, 0x1B);
        }
    } else if kind == Ft_Kind_Luigi {
        if can_use_ranged(cx, fp, target, 60.0)? {
            push_avoid(&mut fp.cpu, 0x26);
        }
    } else if kind == Ft_Kind_Koopa || kind == Ft_Kind_GKoops || kind == Ft_Kind_Yoshi {
        // Grounded: the ranged test (27 / 53 / 16); airborne and off the stage: avoid it.
        let range = if kind == Ft_Kind_Koopa { 27.0 } else if kind == Ft_Kind_GKoops { 53.0 } else { 16.0 };
        if fp.ground_or_air == GA_Ground {
            if can_use_ranged(cx, fp, target, range)? {
                push_avoid(&mut fp.cpu, 0x26);
            }
        } else if !b(fp.cpu.xFA_b5()) {
            push_avoid(&mut fp.cpu, 0x26);
        }
    } else if kind == Ft_Kind_Peach {
        // The same test with range 32 (the C spells it out inline).
        if can_use_ranged(cx, fp, target, 32.0)? {
            push_avoid(&mut fp.cpu, 0x26);
        }
    } else if kind == Ft_Kind_Kirby {
        return Err(format!("ftCo_800B77E8 (kind {kind}) not ported"));
    }
    Ok(())
}

/// `ftCo_800B885C`: the level-gated 'pause' window.
fn ftco_800b885c(fp: &Fighter) -> bool {
    let c = &fp.cpu;
    match c.level {
        0 => c.x80 % 300 <= 240,
        1 => c.x80 % 300 <= 180,
        2 => c.x80 % 300 <= 120,
        3 => c.x80 % 240 <= 120,
        4 => c.x80 % 240 <= 60,
        _ => false,
    }
}

/// `ftCo_800B89CC`: the CPU is allowed to attack from the air.
fn ftco_800b89cc(cx: &Cx, fp: &Fighter) -> bool {
    let tmp = unsafe { &*fp.cpu.x44 };
    if cx.grkind == Gr_Kind_Kongo as i32 && (fp.cur_pos.x > 25.0 || fp.cur_pos.x < -25.0) {
        return false;
    }
    if ftco_800a3200(tmp) != 0 {
        return true;
    }
    if !b(fp.cpu.xF9_b4()) {
        return false;
    }
    if !b(fp.cpu.xFA_b5()) && fp.kind == Ft_Kind_Koopa {
        return false;
    }
    fp.cpu.level >= 4
}

/// `ftCo_800B8A9C`: decide on a melee attack at the target (sets `xA4`).
fn ftco_800b8a9c(cx: &mut Cx, fp: &mut Fighter) -> R<bool> {
    if !b(fp.cpu.xF9_b2()) {
        return Ok(false);
    }
    let target = fp.cpu.x44;
    fp.cpu.xEC = 0;
    fp.cpu.xC8 = 0;
    if target.is_null() {
        fp.cpu.xA4 = 0;
        return Ok(false);
    }
    let t = unsafe { &*target };
    if fp.motion_id == ftCo_MS_Pass as i32 || ftco_800a1c44(t) || ftco_800b885c(fp) || ftco_800a1ca8(fp) {
        fp.cpu.xA4 = 0;
        return Ok(false);
    }
    if t.motion_id >= ftCo_MS_Catch as i32 && t.motion_id <= ftCo_MS_EscapeAir as i32 {
        push_avoid(&mut fp.cpu, 0x28);
        push_avoid(&mut fp.cpu, 0x29);
    } else if t.x34_scale.y > fp.x34_scale.y {
        push_avoid(&mut fp.cpu, 0x28);
        push_avoid(&mut fp.cpu, 0x29);
        if fp.kind == Ft_Kind_Kirby {
            return Err("ftCo_800B8A9C (Kirby) not ported".into());
        }
    }
    ftco_800b77e8(cx, fp)?;
    let kind = fp.kind;
    if fp.ground_or_air == GA_Air as _ {
        if ftco_800b89cc(cx, fp) {
            let l = cx.attack_list(T_AIR, kind);
            let r = ftco_800b4ab0(cx, fp, t, l)?;
            if r != 0 {
                fp.cpu.xA4 = r;
                return Ok(true);
            }
        }
        fp.cpu.xA4 = 0;
        return Ok(false);
    }
    if ftco_800a3134(t) != 0 || ftco_800a3200(t) != 0 {
        if fp.cpu.xC8 < 8 {
            fp.cpu.xA8_array[fp.cpu.xC8 as usize] = 0xA;
            fp.cpu.xC8 += 1;
        }
        if fp.cpu.xC8 < 8 {
            fp.cpu.xA8_array[fp.cpu.xC8 as usize] = 0xE;
            fp.cpu.xC8 += 1;
        }
        let l = cx.attack_list(T_GROUND, kind);
        let r = ftco_800b4ab0(cx, fp, t, l)?;
        if r != 0 {
            fp.cpu.xA4 = r;
            return Ok(true);
        }
        fp.cpu.xA4 = 0;
        return Ok(false);
    }
    if !fp.item_gobj.is_null() {
        return Err("ftCo_800B8A9C (a held weapon) not ported".into());
    }
    if fp.cpu.level > 5 && (fp.motion_id == ftCo_MS_GuardOn as i32 || fp.motion_id == ftCo_MS_Guard as i32) == false {
        // ftCo_800B9F6C(target): the TARGET is shielding.
    }
    if fp.cpu.level > 5 && (t.motion_id == ftCo_MS_GuardOn as i32 || t.motion_id == ftCo_MS_Guard as i32) {
        let l = cx.attack_list(T_SMASH, kind);
        let r = ftco_800b4ab0(cx, fp, t, l)?;
        if r != 0 {
            fp.cpu.xA4 = r;
            return Ok(true);
        }
    }
    let var_r0 = if !fp.item_gobj.is_null() || ftco_800a3200(t) != 0 {
        false
    } else {
        cx.probe(t.cur_pos.x, dadd(5.0, t.cur_pos.y), t.cur_pos.x, dsub(t.cur_pos.y, 1000.0), 0.0)?.is_none()
    };
    if var_r0 {
        let l = cx.attack_list(T_EDGE, kind);
        let r = ftco_800b4ab0(cx, fp, t, l)?;
        if r != 0 {
            fp.cpu.xA4 = r;
            fp.cpu.set_xF8_b7(1);
            return Ok(true);
        }
    }
    let l = cx.attack_list(T_GROUND, kind);
    let r = ftco_800b4ab0(cx, fp, t, l)?;
    if r != 0 {
        fp.cpu.xA4 = r;
        return Ok(true);
    }
    if fp.cpu.x50 != 0 {
        return Err("ftCo_800B5AB0 (a special item target) not ported".into());
    }
    fp.cpu.xA4 = 0;
    Ok(false)
}

/// `ftCo_800B9704`'s caller in B9CBC: the reaction delay (the ported `ftco_800b9704`).
fn react_delay(cx: &mut Cx, fp: &mut Fighter) {
    let r = cx.rand.randf();
    melee_fighter::ftcpu::ftco_800b9704(fp, r);
}

/// `ftCo_800B9CBC`: decide on a ranged attack at the target.
fn ftco_800b9cbc(cx: &mut Cx, fp: &mut Fighter) -> R<bool> {
    let t_ptr = fp.cpu.x44;
    fp.cpu.xA4 = 0;
    if t_ptr.is_null() {
        return Ok(false);
    }
    if !b(fp.cpu.xF9_b5()) {
        return Ok(false);
    }
    if fp.cpu.x34 > 0 {
        fp.cpu.x34 -= 1;
        return Ok(false);
    }
    react_delay(cx, fp);
    let t = unsafe { &*t_ptr };
    // ftCo_800A1BA8: a wall between the fighters.
    let wall = if fp.facing_dir as f64 > 0.0 {
        cx.ground.mp_check_right_wall(fp.cur_pos.x, fp.cur_pos.y, t.cur_pos.x, t.cur_pos.y, -1, -1).is_some()
    } else {
        cx.ground.mp_check_left_wall(fp.cur_pos.x, fp.cur_pos.y, t.cur_pos.x, t.cur_pos.y, -1, -1).is_some()
    };
    if wall {
        return Ok(false);
    }
    let dist = ftco_800a1ab4(fp, t);
    let ax = (t.cur_pos.x - fp.cur_pos.x).abs();
    let ang = cs::ext::lb_8000D008(t.cur_pos.y - fp.cur_pos.y, ax);
    if !fp.item_gobj.is_null() {
        return Err("ftCo_800B9A04 (a held item) not ported".into());
    }
    let r = cx.rand.randf();
    if r as f64 > 0.75 {
        return Ok(false);
    }
    if (dist as f64) < 30.0 {
        return Ok(false);
    }
    let k = fp.kind;
    let var_r3 = if k == Ft_Kind_Ness || k == Ft_Kind_Yoshi {
        return Err("ftCo_800B9790 / ftCo_800B98C8 not ported".into());
    } else if k == Ft_Kind_Samus {
        if fp.ground_or_air == GA_Ground as _ {
            false
        } else if (ang as f64) < -0.7853981573134661 {
            fp.cpu.xA4 = 0x26;
            true
        } else {
            false
        }
    } else {
        false
    };
    if var_r3 {
        return Ok(true);
    }
    if !ftco_800a2c08(fp) {
        return Ok(false);
    }
    if (ang as f64) < 0.08726646192371845 && (ang as f64) > -0.08726646192371845 {
        let l = cx.attack_list(T_RANGED, k).unwrap_or_default();
        fp.cpu.xA4 = ftco_800b6208(cx, &l)?;
        return Ok(fp.cpu.xA4 != 0);
    }
    Ok(false)
}

/// `ftCo_800B7638`: kind-0x10 CPUs approach for a grab.
fn ftco_800b7638(cx: &mut Cx, fp: &Fighter) -> R<i32> {
    let tp = fp.cpu.x44;
    if tp.is_null() {
        return Ok(0);
    }
    let t = unsafe { &*tp };
    if fp.ground_or_air == GA_Air as _ {
        if fp.co_attrs.max_jumps as i32 <= fp.x1968_jumpsUsed as i32 || fp.pos_delta.y as f64 > 0.0 {
            return Ok(0);
        }
    }
    if ((t.cur_pos.y - fp.cur_pos.y) as f64) < 0.0 {
        return Ok(0);
    }
    if t.ground_or_air == GA_Air as _ {
        if cx.ground.mp_check_ceiling(fp.cur_pos.x, fp.cur_pos.y, t.cur_pos.x, t.cur_pos.y, -1, -1).is_some() {
            return Ok(0);
        }
        let _ = ftco_800a1ab4(fp, t);
        if cs::ext::lb_8000D008(t.cur_pos.y - fp.cur_pos.y, (t.cur_pos.x - fp.cur_pos.x).abs()) as f64 > 0.7853981573134661 {
            return Ok(1);
        }
    } else {
        let dist = ftco_800a1ab4(fp, t);
        let ang = cs::ext::lb_8000D008(t.cur_pos.y - fp.cur_pos.y, (t.cur_pos.x - fp.cur_pos.x).abs());
        if dist as f64 > 30.0 && (dist as f64) < 40.0 && ang as f64 > -0.08726646192371845 && (ang as f64) < 0.08726646192371845 {
            return Ok(2);
        }
    }
    Ok(0)
}

/// `ftCo_800B732C`: grab / ledge decisions against the target.
fn ftco_800b732c(cx: &mut Cx, fp: &mut Fighter) -> R<i32> {
    let tp = fp.cpu.x44;
    if tp.is_null() || fp.ground_or_air == GA_Air as _ {
        return Ok(0);
    }
    let t = unsafe { &*tp };
    if ftco_800a3200(t) != 0 {
        if (t.cur_pos.y < fp.cur_pos.y) || cx.ground.mp_check_ceiling(fp.cur_pos.x, fp.cur_pos.y, t.cur_pos.x, t.cur_pos.y, -1, -1).is_some() {
            return Ok(0);
        }
        let _ = ftco_800a1ab4(fp, t);
        if cs::ext::lb_8000D008(t.cur_pos.y - fp.cur_pos.y, (t.cur_pos.x - fp.cur_pos.x).abs()) > 1.3089969f32 {
            return Ok(1);
        }
        return Ok(0);
    }
    if !b(fp.cpu.xF9_b3()) {
        return Ok(0);
    }
    if fp.cpu.kind == 0x10 {
        return ftco_800b7638(cx, fp);
    }
    if !ftco_800a2bd4(fp) || fp.cpu.level < 4 || ftco_800a28d0(cx, fp, 4.0) {
        return Ok(0);
    }
    if t.ground_or_air == GA_Air as _ {
        if t.cur_pos.y < fp.cur_pos.y || cx.ground.mp_check_ceiling(fp.cur_pos.x, fp.cur_pos.y, t.cur_pos.x, t.cur_pos.y, -1, -1).is_some() {
            return Ok(0);
        }
        let d = ftco_800a1ab4(fp, t);
        if cs::ext::lb_8000D008(t.cur_pos.y - fp.cur_pos.y, (t.cur_pos.x - fp.cur_pos.x).abs()) as f64 > 0.7853981573134661 && (d as f64) < 30.0 {
            return Ok(1);
        }
    } else {
        if fp.cpu.x7C % 60 != 0 {
            return Ok(0);
        }
        let r = cx.rand.randf();
        if r as f64 > 0.1 {
            return Ok(0);
        }
        let d = ftco_800a1ab4(fp, t);
        let a = cs::ext::lb_8000D008(t.cur_pos.y - fp.cur_pos.y, (t.cur_pos.x - fp.cur_pos.x).abs());
        if d as f64 > 30.0 && (d as f64) < 40.0 && a as f64 > -0.08726646192371845 && (a as f64) < 0.08726646192371845 {
            return Ok(2);
        }
    }
    Ok(0)
}

/// `ftCo_800B2790`: run the chosen behaviour when the script is idle.
fn ftco_800b2790(cx: &mut Cx, me: *mut Fighter) -> R {
    let fp = unsafe { &mut *me };
    if !fp.cpu.csP.is_null() || fp.cpu.command_duration != 0 {
        return Ok(());
    }
    fp.cpu.x80 += 1;
    fp.cpu.set_xF8_b7(0);
    cs::ftco_800b462c(fp);
    ftco_800adc28(fp)?;
    match fp.cpu.x18 {
        1 => ftco_800aba34(cx, fp)?,
        2 => ftco_800b658c(cx, fp)?,
        3 => ftco_800b9020(cx, fp)?,
        7 => ftco_800ba9a0(cx, fp)?,
        c => return Err(format!("ftCo_800B2790: behaviour {c} not ported")),
    }
    cs::ftco_800b49f4(fp);
    Ok(())
}

/// `ftCo_CpuClearTargetAndFinish`.
fn clear_target_and_finish(fp: &mut Fighter) {
    fp.cpu.x44 = std::ptr::null_mut();
    fp.cpu.x18 = fp.cpu.x1C;
    ftco_800a0c8c(fp);
}

/// `ftCo_800B9020`: behaviour 3, pick and run an attack against the target.
fn ftco_800b9020(cx: &mut Cx, fp: &mut Fighter) -> R {
    if fp.cpu.x44.is_null() {
        clear_target_and_finish(fp);
        return Ok(());
    }
    let k = fp.kind;
    if k == Ft_Kind_Ness || k == Ft_Kind_Yoshi || k == Ft_Kind_Zelda {
        return Err(format!("ftCo_800B9020 (kind {k}) not ported"));
    }
    if ftco_800b630c(fp)? {
        neutral_stick(fp);
        ftco_800a0c8c(fp);
        fp.cpu.xA4 = 0;
    } else if fp.cpu.xA4 != 0 {
        let a4 = fp.cpu.xA4;
        cx.run_cmdscript(fp, a4);
        fp.cpu.xA4 = 0;
    } else if ftco_800b9cbc(cx, fp)? {
        if fp.cpu.xA4 != 0 {
            let a4 = fp.cpu.xA4;
            cx.run_cmdscript(fp, a4);
            fp.cpu.xA4 = 0;
            fp.cpu.x90 += 1;
        } else {
            clear_target_and_finish(fp);
        }
    } else {
        clear_target_and_finish(fp);
    }
    Ok(())
}

/// `ftCo_CpuFireBlaster` (a static inline of ftcpuattack.c): down + B, then a level-scaled wait.
fn fire_blaster(cx: &mut Cx, fp: &mut Fighter) {
    let delay = 9 - fp.cpu.level;
    cmd1(fp, CpuCmd_SetLstickX, 0);
    cmd1(fp, CpuCmd_SetLstickY, -0x50);
    cmd0(fp, CpuCmd_PressB);
    cmd1(fp, CpuCmd_WaitFor, 1);
    cmd1(fp, CpuCmd_SetLstickY, 0);
    if delay != 0 {
        let r = cx.rand.randf();
        cmd1(fp, CpuCmd_WaitFor, delay * ((4.0f32 * r) as i32 + 4));
    }
    done(fp);
}

/// `ftCo_CpuRetapR`.
fn retap_r(fp: &mut Fighter) {
    cmd0(fp, CpuCmd_ReleaseR);
    cmd1(fp, CpuCmd_WaitFor, 1);
    cmd0(fp, CpuCmd_PressR);
    cmd1(fp, CpuCmd_WaitFor, 1);
    cmd0(fp, CpuCmd_ReleaseR);
    done(fp);
}

/// `ftCo_800BA9A0`: behaviour 7, answer the threat `ftCo_800BB9B4` found (`xF8_b12`).
fn ftco_800ba9a0(cx: &mut Cx, fp: &mut Fighter) -> R {
    let m = fp.motion_id;
    if m == ftCo_MS_EscapeF as i32 || m == ftCo_MS_EscapeB as i32 || m == ftCo_MS_EscapeAir as i32 || m == ftCo_MS_EscapeN as i32 {
        ftco_800a0c8c(fp);
        return Ok(());
    }
    if m == 0xB2 {
        ftco_800b9f90(cx, fp);
        return Ok(());
    }
    let t = fp.cpu.xF8_b12();
    if t == 0 {
        fp.cpu.x18 = fp.cpu.x1C;
        neutral_stick(fp);
        cmd0(fp, CpuCmd_ReleaseR);
        cmd0(fp, CpuCmd_ReleaseB);
        ftco_800a0c8c(fp);
        return Ok(());
    }
    let fox = fp.kind == Ft_Kind_Fox || fp.kind == Ft_Kind_Falco;
    if fp.ground_or_air == GA_Air {
        if fox {
            if t == 2 {
                fire_blaster(cx, fp);
            } else {
                retap_r(fp);
            }
        } else if t == 3 {
            return Err("ftCo_800BA080 not ported".into());
        } else {
            retap_r(fp);
        }
    } else if t == 2 {
        let ip = unsafe { &*fp.cpu.xF4 };
        let k = ip.kind;
        if k == It_Kind_Flipper as _ || k == It_Kind_TaruCann as _ || k == It_Kind_DKinoko as _ {
            return Err("ftCo_800BA9A0: Flipper / barrel cannon / mushroom not ported".into());
        }
        if fox {
            let r = cx.rand.randf();
            if r as f64 > 0.5 {
                fire_blaster(cx, fp);
            } else {
                ftco_800b9f90(cx, fp);
            }
        } else {
            ftco_800b9f90(cx, fp);
        }
    } else if t == 3 {
        return Err("ftCo_800BA080 not ported".into());
    } else if fp.cpu.xF0.is_null() {
        ftco_800b9f90(cx, fp);
    } else {
        let o = unsafe { &*fp.cpu.xF0 };
        let d = fp.cur_pos.x - o.cur_pos.x;
        let a = if d < 0.0 { -d } else { d };
        if a as f64 > 30.0 {
            if d as f64 > 0.0 { ftco_800ba160(fp) } else { ftco_800ba224(fp) }
        } else if d as f64 > 0.0 {
            ftco_800ba224(fp)
        } else {
            ftco_800ba160(fp)
        }
    }
    Ok(())
}

/// `ftCo_800B658C`: behaviour 2, attack the target.
fn ftco_800b658c(cx: &mut Cx, fp: &mut Fighter) -> R {
    let t_ptr = fp.cpu.x44;
    if t_ptr.is_null() {
        clear_target_and_finish(fp);
        return Ok(());
    }
    if b(fp.cpu.xF8_b6()) && fp.ground_or_air == GA_Ground as _ {
        let r = cx.rand.randf();
        if r < 0.05f32 * fp.cpu.level as f32 {
            fp.cpu.set_xF8_b6(0);
            if ftco_800ba2e8(cx, fp, unsafe { &*t_ptr }) {
                return Ok(());
            }
        }
    }
    let k = fp.kind;
    if k == Ft_Kind_Ness || k == Ft_Kind_Yoshi || k == Ft_Kind_Samus || k == Ft_Kind_Donkey || k == Ft_Kind_Zelda {
        return Err(format!("ftCo_800B658C (kind {k}) not ported"));
    }
    if ftco_800b630c(fp)? {
        ftco_800a0c8c(fp);
        fp.cpu.xA4 = 0;
        return Ok(());
    }
    if fp.cpu.xA4 == 0 {
        if fp.ground_or_air == GA_Ground as _ && !ftco_800a2c08(fp) {
            ftco_800a0098(fp);
            return Ok(());
        }
        if !ftco_800b8a9c(cx, fp)? {
            clear_target_and_finish(fp);
            return Ok(());
        }
    }
    ftco_800b63d8(cx, fp);
    let a4 = fp.cpu.xA4;
    cx.run_cmdscript(fp, a4);
    fp.cpu.xA4 = 0;
    Ok(())
}

/// `ftCo_800ADC28`: keep charging specials (Donkey Kong, Samus, Mewtwo, Kirby).
fn ftco_800adc28(fp: &Fighter) -> R {
    if fp.cpu.x18 == 0xA {
        return Ok(());
    }
    let k = fp.kind;
    let m = fp.motion_id;
    let charging = if k == Ft_Kind_Donkey {
        m == ftDk_MS_SpecialNLoop as i32
    } else if k == Ft_Kind_Samus {
        m == ftSs_MS_SpecialNHold as i32
    } else if k == Ft_Kind_Mewtwo {
        m == ftMt_MS_SpecialNLoop as i32 || m == ftMt_MS_SpecialNLoopFull as i32
    } else {
        // Kirby copying one of them.
        k == Ft_Kind_Kirby
    };
    if charging {
        return Err("ftCo_800ADC28 (charge special) not ported".into());
    }
    Ok(())
}

/// `ftCo_800ABA34`: behaviour 1, the default (approach and fight).
fn ftco_800aba34(cx: &mut Cx, fp: &mut Fighter) -> R {
    let kind = fp.cpu.kind;
    if kind == 11 || kind == 12 || kind == 16 {
        return Err("ftCo_800ABA34 (cpu kind 11/12/16) not ported".into());
    }
    if fp.kind == Ft_Kind_Zelda || fp.kind == Ft_Kind_Seak {
        if b(fp.cpu.xF8_b5()) {
            return Err("ftCo_800ABA34 (Sheik / Zelda special) not ported".into());
        }
    }
    let _ = cx;
    let me: *mut Fighter = fp;
    if ftco_800a5ce0(cx, me)?.is_null() && fp.motion_id == ftCo_MS_RebirthWait as i32 {
        ftco_800a08f0(fp);
        return Ok(());
    }
    if fp.ground_or_air == GA_Air as _ {
        return ftco_800a9cb4(cx, fp);
    }
    ftco_800ab224(cx, fp)
}

/// `ftCo_800A3234`: an airborne fighter that should use its jump / up-B to get back.
fn ftco_800a3234(cx: &mut Cx, fp: &Fighter) -> R<bool> {
    let d = &fp.cpu;
    if b(d.xFA_b6()) {
        return Ok(fp.pos_delta.y < 0.1);
    }
    if fp.ground_or_air != GA_Air as _ {
        return Ok(false);
    }
    if fp.pos_delta.y as f64 > 0.0 || fp.motion_id == 0xF4 {
        return Ok(false);
    }
    if fp.kind == Ft_Kind_Yoshi && fp.x34_scale.y > 1.0 && d.x7C < 0x384 {
        return Ok(false);
    }
    if cs::ext::lb_8000D008(d.x54.y - fp.cur_pos.y, (d.x54.x - fp.cur_pos.x).abs()) < -1.3089969f32 {
        return Ok(false);
    }
    let wall = if fp.facing_dir > 0.0 { fp.coll_data.env_flags & Collide_LeftWallMask != 0 } else { fp.coll_data.env_flags & Collide_RightWallMask != 0 };
    if wall {
        return Ok(true);
    }
    let (bx, by) = (fp.coll_data.ecb.bottom.x, fp.coll_data.ecb.bottom.y);
    let y = fp.coll_data.last_pos.y + by;
    if cx.probe(fp.coll_data.last_pos.x + bx, fp.coll_data.last_pos.y + by, fp.coll_data.cur_pos.x + bx, fp.coll_data.cur_pos.y + by, 0.0)?.is_some() {
        return Ok(false);
    }
    Ok(d.x54.y > y)
}

fn neutral_stick(fp: &mut Fighter) {
    // ftCo_CpuSetNeutralStick
    cs::ftco_800b46b8(fp, CpuCmd_SetLstickX as u8, 0);
    cs::ftco_800b46b8(fp, CpuCmd_SetLstickY as u8, 0);
}

fn finish_neutral_y(fp: &mut Fighter) {
    cs::ftco_800b46b8(fp, CpuCmd_SetLstickY as u8, 0);
    cs::ftco_800b463c(fp, CpuCmd_Done as u8);
}

/// `ftCo_800A9CB4`: airborne approach to the destination.
fn ftco_800a9cb4(cx: &mut Cx, fp: &mut Fighter) -> R {
    if ftco_800a1ca8(fp) {
        neutral_stick(fp);
        cs::ftco_800b463c(fp, CpuCmd_Done as u8);
        return Ok(());
    }
    if ftco_800a3234(cx, fp)? {
        if fp.co_attrs.max_jumps as i32 > fp.x1968_jumpsUsed as i32 {
            return Err("ftCo_CpuUseAvailableJump not ported".into());
        }
        if cx.grkind == Gr_Kind_Shrine as i32 || cx.grkind == Gr_Kind_Fourside as i32 {
            return Err("ftCo_800A9CB4 (stage up-B) not ported".into());
        }
        if !b(fp.cpu.xFA_b6()) && (fp.x34_scale.y < 1.0 || cx.grkind == Gr_Kind_Icemt as i32 || cx.grkind == Gr_Kind_RCruise as i32) {
            return Err("ftCo_800A96B8 (recovery special) not ported".into());
        }
    }
    if b(fp.cpu.xFA_b6()) {
        cs::ftco_800b46b8(fp, CpuCmd_SetLstickX as u8, 0x81);
        finish_neutral_y(fp);
        return Ok(());
    }
    let nz = |x: f32| !(x < 0.00001 && x > -0.00001);
    let dx = fp.cpu.x54.x - fp.cur_pos.x;
    let x_delta_abs = dx.abs();
    let x_time = if nz(fp.pos_delta.x) {
        dx / fp.pos_delta.x
    } else if x_delta_abs as f64 > 0.0 {
        -1.0f32
    } else {
        0.0f32
    };
    let gravity = fp.co_attrs.gravity;
    let terminal_time = if !nz(gravity) { 1000.0f32 } else { -(-fp.co_attrs.terminal_velocity - fp.pos_delta.y) / gravity };
    let y_pos = if terminal_time <= 0.0 {
        // FMA: pos_delta.y * x_time + cur_pos.y
        fp.pos_delta.y.mul_add(x_time, fp.cur_pos.y)
    } else if x_time < terminal_time {
        let s = sqrtf(x_time);
        // cur_y + (vy * x_time - 0.5 * (g * s)) in double after the first product.
        (fp.cur_pos.y as f64 + (-0.5f64).mul_add((gravity * s) as f64, (fp.pos_delta.y * x_time) as f64)) as f32
    } else {
        let s = sqrtf(terminal_time);
        let xd = x_time - terminal_time;
        (fp.cur_pos.y as f64 + ((-0.5f64).mul_add((gravity * s) as f64, (fp.pos_delta.y * terminal_time) as f64) - (xd * fp.co_attrs.terminal_velocity) as f64)) as f32
    };
    if (x_time as f64) < 0.0 {
        cs::ftco_800b46b8(fp, CpuCmd_LstickXTowardDestination as u8, 0x7F);
    } else if y_pos < fp.cpu.x54.y {
        let mut clamp = ((fp.cpu.x54.y - y_pos) as i32) * 2;
        if clamp > 0x20 {
            clamp = 0x20;
        }
        cs::ftco_800b4778(fp, CpuCmd_LstickXTowardDestinationClamped as u8, clamp as u8, 0x7F);
    } else {
        cs::ftco_800b46b8(fp, CpuCmd_SetLstickX as u8, 0);
    }
    finish_neutral_y(fp);
    Ok(())
}

/// `ftCo_800BB768`: is `o` doing a move that comes down on `fp` (Bowser / Yoshi down-B, Bowser's
/// up smash, Samus's up-B)?
fn ftco_800bb768(fp: &Fighter, o: &Fighter) -> R<bool> {
    let m = o.motion_id;
    let k = o.kind;
    if k == Ft_Kind_GKoops || k == Ft_Kind_Koopa {
        if m == ftKp_MS_SpecialLw as i32 || m == ftKp_MS_SpecialAirLw as i32 || m == ftCo_MS_AttackHi4 as i32 {
            return Err("ftCo_800BB768 (Bowser's down-B / up smash) not ported".into());
        }
    } else if k == Ft_Kind_Yoshi {
        if m == ftYs_MS_SpecialLw as i32 || m == ftYs_MS_SpecialAirLw as i32 {
            return Err("ftCo_800BB768 (Yoshi's down-B) not ported".into());
        }
    } else if k == Ft_Kind_Samus && m == ftSs_MS_SpecialHi as i32 && o.cur_pos.y < fp.cur_pos.y && (ftco_800a1ab4(fp, o) as f64) < 30.0 {
        return Ok(true);
    }
    Ok(false)
}

/// `ftCo_800BB9B4`: scan the other fighters' and the items' hitboxes for one about to hit
/// (`xF8_b12` 1..3: what kind of threat, `xF0` / `xF4`: its source).
fn ftco_800bb9b4(cx: &mut Cx, me: *mut Fighter) -> R {
    let fp = unsafe { &mut *me };
    fp.cpu.set_xF8_b12(0);
    fp.cpu.xF0 = core::ptr::null_mut();
    fp.cpu.xF4 = core::ptr::null_mut();
    if ftco_800a1c44(fp) || fp.kind == Ft_Kind_GKoops {
        return Ok(());
    }
    for i in 0..cx.fps.len() {
        let cur = cx.fps[i];
        if cur == me {
            continue;
        }
        let o = unsafe { &*cur };
        if ftco_is_ally(fp, o) {
            continue;
        }
        // ftCo_800BB104: any enabled hitbox of the other fighter.
        for h in o.x914.iter() {
            if h.state != HitCapsule_Disabled as _ {
                return Err("ftCo_800BB104 (an active fighter hitbox) not ported".into());
            }
        }
        // ftCo_800BB768: special moves that threaten without a hitbox.
        if ftco_800bb768(fp, o)? {
            // xF8_b12 = 1, xF0 = the fighter: the reaction (behaviour 7) reads it.
            fp.cpu.set_xF8_b12(1);
            fp.cpu.xF0 = cur;
            return Ok(());
        }
    }
    let half = 0.5f32 * fp.cpu.x568;
    let mut sp24 = Vec3 { x: fp.cur_pos.x, y: fp.cur_pos.y + half, z: fp.cur_pos.z };
    for k in 0..cx.items.len() {
        let it = cx.items[k];
        use super::fox::ItemClass as C;
        match it.class {
            // It_Kind_Yaku: no hitbox.
            C::GreensBlock => {}
            // The blaster gun and the held needles: Disabled hit capsules (it_8026C1B4 false).
            C::FoxGun { .. } | C::SeakHeld { .. } => {}
            C::FoxLaser { .. } => {
                // it_8026C1B4: any hit capsule not Disabled.
                let r = ftco_800bb220(cx, me, &it, &mut sp24, half)?;
                fp.cpu.set_xF8_b12(r as _);
                if r != 0 {
                    fp.cpu.xF4 = item_stub(fp.player_id as usize, &it);
                    return Ok(());
                }
            }
            C::SeakThrown { .. } => return Err("ftCo_800BB220 (Sheik's needle) not ported".into()),
        }
    }
    Ok(())
}

/// A stand-in `Item` for `fp->cpu.xF4` (the CPU reads the threatening item's `kind` and `pos`): one per
/// player, kept alive for the run.
fn item_stub(slot: usize, it: &super::fox::SimItem) -> *mut Item {
    use std::cell::RefCell;
    thread_local! {
        static STUBS: RefCell<std::vec::Vec<Box<Item>>> = const { RefCell::new(std::vec::Vec::new()) };
    }
    STUBS.with(|s| {
        let mut s = s.borrow_mut();
        while s.len() <= slot {
            s.push(Box::new(Item::zeroed()));
        }
        let ip = &mut *s[slot];
        ip.kind = it.kind as _;
        ip.pos = Vec3 { x: it.pos.x, y: it.pos.y, z: it.pos.z };
        &mut *ip as *mut Item
    })
}

/// `ftCo_800BB220`: will the laser `it` reach `fp` (`arg2`, radius `arg3`) in a level-scaled number of
/// frames? 0: no, 2: yes. (The Ness reflect branches are not ported.)
fn ftco_800bb220(cx: &mut Cx, me: *mut Fighter, it: &super::fox::SimItem, arg2: &mut Vec3, arg3: f32) -> Result<i32, String> {
    use super::fox::ItemClass as C;
    let fp = unsafe { &mut *me };
    let C::FoxLaser { owner, hits, .. } = it.class else {
        return Err("ftCo_800BB220 (an item that is not a laser) not ported".into());
    };
    if hits.state[..hits.n].iter().all(|&s| s == 0) {
        return Ok(0);
    }
    if !hits.valid {
        return Err("ftCo_800BB220: hit state of a laser made by the s_link 9 accessory not modelled".into());
    }
    if cx.fps[owner] == me {
        return Ok(0);
    }
    // ftLib_80086960(ip->owner) (a fighter) && ftCo_IsAlly.
    if ftco_is_ally(fp, unsafe { &*cx.fps[owner] }) {
        return Ok(0);
    }
    if it.kind == It_Kind_Star as i32 || it.kind == It_Kind_Unk4 as i32 || ftco_800a5980_motion(it.kind) {
        return Ok(0);
    }
    let level = fp.cpu.level;
    let count: i32 = if level < 3 {
        (20.0f32 * cx.rand.randf()) as i32 + 10
    } else if level < 6 {
        (10.0f32 * cx.rand.randf()) as i32 + 5
    } else if level < 9 {
        (5.0f32 * cx.rand.randf()) as i32 + 3
    } else {
        1
    };
    let c = count as f32;
    let mut dst = Vec3 {
        x: melee_core::fp::fmadds(fp.pos_delta.x, c, arg2.x),
        y: melee_core::fp::fmadds(fp.pos_delta.y, c, arg2.y),
        z: melee_core::fp::fmadds(fp.pos_delta.z, c, arg2.z),
    };
    if fp.kind == Ft_Kind_Ness && level > 3 {
        return Err("ftCo_800BB220 (Ness reflect) not ported".into());
    }
    for i in 0..4 {
        // state not Disabled / Enabled, x43_b2 clear, element not Inert (the laser's is 0).
        if hits.state[i] == 0 || hits.state[i] == 1 {
            continue;
        }
        // lbColl_8000ACFC: the victims list holds nobody (hits are not ported).
        let mut x4c = Vec3 { x: hits.x4c[i].x, y: hits.x4c[i].y, z: hits.x4c[i].z };
        let x58 = hits.x58[i];
        // ftCo_CpuPredictHitboxPosition.
        let mut pred = Vec3 {
            x: melee_core::fp::fmadds(x4c.x - x58.x, c, x4c.x),
            y: melee_core::fp::fmadds(x4c.y - x58.y, c, x4c.y),
            z: melee_core::fp::fmadds(x4c.z - x58.z, c, x4c.z),
        };
        let mut a = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut b2 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let hit = unsafe { melee_runtime::tu::lbcollision::lbColl_80006094(&mut x4c, &mut pred, arg2, &mut dst, &mut a, &mut b2, super::fox::LASER_HIT_SCALE, arg3) };
        if hit {
            return Ok(2);
        }
    }
    Ok(0)
}

// ---- command-script helpers (the C's static inlines) ----

fn cmd1(fp: &mut Fighter, c: u32, a: i32) {
    cs::ftco_800b46b8(fp, c as u8, a as u8);
}
fn cmd0(fp: &mut Fighter, c: u32) {
    cs::ftco_800b463c(fp, c as u8);
}
fn cmd2(fp: &mut Fighter, c: u32, a: i32, b2: i32) {
    cs::ftco_800b4778(fp, c as u8, a as u8, b2 as u8);
}
fn done(fp: &mut Fighter) {
    cmd0(fp, CpuCmd_Done);
}

/// `ftCo_CpuTurnAwayFromLedge`.
fn turn_away_from_ledge(fp: &mut Fighter) {
    cmd1(fp, CpuCmd_SetLstickY, 0);
    cmd1(fp, CpuCmd_SetLstickX, 0);
    cmd1(fp, CpuCmd_WaitFor, 1);
    cmd1(fp, CpuCmd_LstickXForward, 0x7F);
    cmd1(fp, CpuCmd_WaitFor, 1);
    cmd1(fp, CpuCmd_SetLstickX, 0);
    done(fp);
}

/// `ftCo_CpuTurnAround`.
fn turn_around(fp: &mut Fighter) {
    neutral_stick(fp);
    cmd1(fp, CpuCmd_WaitFor, 1);
    cmd1(fp, CpuCmd_LstickXForward, 0xB0);
    cmd1(fp, CpuCmd_WaitFor, 1);
    cmd1(fp, CpuCmd_SetLstickY, 0);
    cmd1(fp, CpuCmd_WaitFor, 10);
    cmd1(fp, CpuCmd_SetLstickX, 0);
    done(fp);
}

fn is_in_teeter(fp: &Fighter) -> bool {
    fp.motion_id == ftCo_MS_Ottotto as i32 || fp.motion_id == ftCo_MS_OttottoWait as i32
}

/// `ftCo_800AA320`: stick step and clamp by level.
fn ftco_800aa320(fp: &Fighter) -> (i32, i32) {
    if fp.kind == Ft_Kind_Nana {
        return (0x40, 0x7F);
    }
    match fp.cpu.level {
        0 => (1, 0x43),
        1 => (2, 0x48),
        2 => (2, 0x4D),
        3 => (2, 0x52),
        4 => (2, 0x57),
        5 => (2, 0x5C),
        6 => (4, 0x61),
        7 => (4, 0x6B),
        8 => (8, 0x75),
        _ => (8, 0x7F),
    }
}

/// `ftCo_800A0148`: jump to get back up to the destination (the generic jump).
fn ftco_800a0148(cx: &Cx, fp: &mut Fighter) {
    if fp.cur_pos.y + fp.cpu.x558 > cx.top() {
        neutral_stick(fp);
        done(fp);
    } else if ftco_800a1ca8(fp) {
        neutral_stick(fp);
        double_jump_after_delay(fp);
    } else if ((fp.cpu.x54.x - fp.cur_pos.x).abs() as f64) > 30.0 {
        cmd0(fp, CpuCmd_ReleaseY);
        cmd1(fp, CpuCmd_WaitFor, 1);
        cmd1(fp, CpuCmd_LstickTowardDestination, 0x7F);
        cmd0(fp, CpuCmd_PressY);
        cmd1(fp, CpuCmd_WaitFor, 10);
        cmd0(fp, CpuCmd_ReleaseY);
        cmd1(fp, CpuCmd_WaitFor, 1);
        done(fp);
    } else {
        cmd0(fp, CpuCmd_ReleaseY);
        neutral_stick(fp);
        cmd1(fp, CpuCmd_WaitFor, 1);
        cmd0(fp, CpuCmd_PressY);
        cmd1(fp, CpuCmd_WaitFor, 10);
        cmd0(fp, CpuCmd_ReleaseY);
        cmd1(fp, CpuCmd_WaitFor, 1);
        done(fp);
    }
}

/// `ftCo_CpuDoubleJumpAfterDelay`.
fn double_jump_after_delay(fp: &mut Fighter) {
    cmd1(fp, CpuCmd_WaitFor, 10);
    cmd0(fp, CpuCmd_ReleaseY);
    cmd1(fp, CpuCmd_WaitFor, 1);
    cmd0(fp, CpuCmd_PressY);
    cmd1(fp, CpuCmd_WaitFor, 10);
    cmd0(fp, CpuCmd_ReleaseY);
    cmd1(fp, CpuCmd_WaitFor, 1);
    done(fp);
}

/// `ftCo_800A0384`: jump forward.
fn ftco_800a0384(cx: &Cx, fp: &mut Fighter) {
    if fp.cur_pos.y + fp.cpu.x558 > cx.top() {
        neutral_stick(fp);
        done(fp);
    } else if ftco_800a1ca8(fp) {
        neutral_stick(fp);
        double_jump_after_delay(fp);
    } else {
        cmd0(fp, CpuCmd_ReleaseY);
        cmd1(fp, CpuCmd_WaitFor, 1);
        cmd1(fp, CpuCmd_LstickXForward, 0x7F);
        cmd0(fp, CpuCmd_PressY);
        cmd1(fp, CpuCmd_WaitFor, 10);
        cmd0(fp, CpuCmd_ReleaseY);
        cmd1(fp, CpuCmd_WaitFor, 1);
        done(fp);
    }
}

/// `ftCo_800A08F0`: drop through a platform / wait (level-scaled).
fn ftco_800a08f0(fp: &mut Fighter) {
    neutral_stick(fp);
    let m = fp.motion_id;
    if m == ftCo_MS_SquatRv as i32 || m == ftCo_MS_Squat as i32 || m == ftCo_MS_SquatWait as i32 {
        cmd1(fp, CpuCmd_WaitFor, 1);
        done(fp);
        return;
    }
    if fp.kind == Ft_Kind_Nana {
        cmd1(fp, CpuCmd_WaitFor, 1);
        cmd1(fp, CpuCmd_SetLstickY, -0x7F);
        cmd1(fp, CpuCmd_WaitFor, 5);
        neutral_stick(fp);
        cmd1(fp, CpuCmd_WaitFor, 1);
        done(fp);
        return;
    }
    let w = match fp.cpu.level {
        0 => Some(45),
        1 | 2 => Some(30),
        3 | 4 => Some(15),
        5 | 6 => Some(10),
        7 | 8 => Some(5),
        9 => Some(1),
        _ => None,
    };
    if let Some(w) = w {
        cmd1(fp, CpuCmd_WaitFor, w);
    }
    cmd1(fp, CpuCmd_SetLstickY, -0x7F);
    cmd1(fp, CpuCmd_WaitFor, 5);
    neutral_stick(fp);
    cmd1(fp, CpuCmd_WaitFor, 1);
    done(fp);
}

/// `ftCo_800A28D0(fp, arg)`: the fighter is within `5 * arg` of the island end it faces.
fn ftco_800a28d0(cx: &Cx, fp: &Fighter, arg: f32) -> bool {
    let isl = island_of(cx, fp);
    if isl == -1 {
        return false;
    }
    let s = cx.ground.islands.segs[isl as usize];
    let off = if fp.facing_dir as f64 > 0.0 { (s.x14.x - fp.cur_pos.x).abs() } else { (s.x8.x - fp.cur_pos.x).abs() };
    (off as f64) < 5.0 * arg as f64
}

/// A grounded fighter's island index (`mpIsland_8005AB54`; -1 = NULL), air = -1.
fn island_of(cx: &Cx, fp: &Fighter) -> i32 {
    if fp.ground_or_air == GA_Air as _ {
        return -1;
    }
    cx.ground.mp_island_8005ab54(fp.coll_data.floor.index)
}

/// `ftCo_800A2BD4`: the destination is in front of the fighter.
fn ftco_800a2bd4(fp: &Fighter) -> bool {
    (fp.facing_dir * (fp.cpu.x54.x - fp.cur_pos.x)) as f64 >= 0.0
}

/// `ftCo_800A2C08`: the target is in front of (or on top of) the fighter.
fn ftco_800a2c08(fp: &Fighter) -> bool {
    let o = fp.cpu.x44;
    if o.is_null() {
        return true;
    }
    let d = unsafe { (*o).cur_pos.x } - fp.cur_pos.x;
    if (d.abs() as f64) < 1.0 {
        return true;
    }
    (d * fp.facing_dir) as f64 >= 0.0
}

/// `ftCo_800A2170`: both fighters are grounded on the same island.
fn ftco_800a2170(cx: &Cx, a: &Fighter, c: &Fighter) -> bool {
    if a.ground_or_air == GA_Air as _ || c.ground_or_air == GA_Air as _ {
        return false;
    }
    let i = cx.ground.mp_island_8005ab54(a.coll_data.floor.index);
    i != -1 && cx.ground.mp_island_8005ab54(c.coll_data.floor.index) == i
}

fn wall_mask(fp: &Fighter) -> bool {
    fp.coll_data.env_flags & (Collide_LeftWallMask | Collide_RightWallMask) != 0
}

/// `ftCo_800AABC8`: walk toward the destination (turning back at a teeter).
fn ftco_800aabc8(fp: &mut Fighter) {
    if is_in_teeter(fp) {
        cmd1(fp, CpuCmd_SetLstickY, 0);
        cmd1(fp, CpuCmd_SetLstickX, 0);
        cmd1(fp, CpuCmd_WaitFor, 1);
        cmd1(fp, CpuCmd_LstickXForward, 0x7F);
        cmd1(fp, CpuCmd_WaitFor, 1);
        cmd1(fp, CpuCmd_SetLstickX, 0);
        done(fp);
    } else {
        let (stick, clamp) = ftco_800aa320(fp);
        cmd1(fp, CpuCmd_SetLstickY, 0);
        cmd2(fp, CpuCmd_LstickForwardClamped, stick, clamp);
        done(fp);
    }
}

/// `ftCo_800AA844`: the destination is level or slightly below.
fn ftco_800aa844(cx: &mut Cx, fp: &mut Fighter) -> R {
    if is_in_teeter(fp) {
        turn_away_from_ledge(fp);
        return Ok(());
    }
    let (stick, clamp) = ftco_800aa320(fp);
    if ftco_800a28d0(cx, fp, 1.0) {
        let on_platform = fp.coll_data.floor.index != -1 && cx.ground.flags(fp.coll_data.floor.index) & LINE_FLAG_PLATFORM != 0;
        if ftco_800a1ca8(fp)
            && on_platform
            && cx.probe(fp.cur_pos.x, fp.cur_pos.y - 10.0f32, fp.cur_pos.x, fp.cur_pos.y - 1000.0f32, 0.0)?.is_some()
        {
            ftco_800a08f0(fp);
        } else if fp.cpu.x54.y > fp.cur_pos.y || (fp.cpu.x54.x - fp.cur_pos.x).abs() as f64 > 40.0 || wall_mask(fp) {
            ftco_800a0148(cx, fp);
        } else {
            cmd1(fp, CpuCmd_SetLstickY, 0);
            cmd2(fp, CpuCmd_LstickForwardClamped, stick, clamp);
            done(fp);
        }
    } else if wall_mask(fp) {
        ftco_800a0148(cx, fp);
    } else {
        cmd1(fp, CpuCmd_SetLstickY, 0);
        cmd2(fp, CpuCmd_LstickForwardClamped, stick, clamp);
        done(fp);
    }
    Ok(())
}

/// `ftCo_800AACD0`: the destination is far below.
fn ftco_800aacd0(cx: &mut Cx, fp: &mut Fighter) -> R {
    if is_in_teeter(fp) {
        turn_away_from_ledge(fp);
        return Ok(());
    }
    let (stick, clamp) = ftco_800aa320(fp);
    if fp.coll_data.floor.index != -1 && cx.ground.flags(fp.coll_data.floor.index) & LINE_FLAG_PLATFORM != 0 {
        ftco_800a08f0(fp);
        return Ok(());
    }
    let isl = cx.ground.mp_island_8005ab54(fp.coll_data.floor.index);
    if isl != -1 {
        let (l, r) = cx.ground.mp_island_8005ace8(isl, true, true);
        let (l, r) = (l.unwrap(), r.unwrap());
        if (fp.cpu.x54.x < l.x || fp.cpu.x54.x > r.x) && !ftco_800a2bd4(fp) {
            turn_around(fp);
            return Ok(());
        }
    }
    if cx.grkind != Gr_Kind_Icemt as i32 && wall_mask(fp) {
        ftco_800a0148(cx, fp);
        return Ok(());
    }
    cmd1(fp, CpuCmd_SetLstickY, 0);
    cmd2(fp, CpuCmd_LstickForwardClamped, stick, clamp);
    done(fp);
    Ok(())
}

/// `ftCo_800AAF48`: Pokemon Stadium-era hop onto a higher island (Shrine only).
fn ftco_800aaf48(cx: &Cx, fp: &Fighter) -> R<bool> {
    if fp.cpu.x60 != 0 || cx.grkind != Gr_Kind_Shrine as i32 {
        return Ok(false);
    }
    Err("ftCo_800AAF48 (Shrine) not ported".into())
}

/// `ftCo_800AA42C_inline0`.
fn aa42c_inline0(cx: &Cx, clamp: f32, dist: f32, near: f32, far: f32) -> i32 {
    if dist > far {
        return clamp as i32;
    }
    if dist < near {
        return 0;
    }
    let mut raw = 127.0f32 * cx.h_deadzone;
    if raw > clamp {
        raw = clamp;
    }
    if raw < 80.0 {
        raw = 80.0;
    }
    let scale = (dist - near) / (far - near);
    // FMA: raw + scale * (clamp - raw)
    let tmp = scale.mul_add(clamp - raw, raw);
    let mut new_clamp = tmp as i32;
    if new_clamp as f32 > clamp {
        new_clamp = clamp as i32;
    }
    new_clamp
}

/// `ftCo_800AA42C`: walk / run along the island to the destination.
fn ftco_800aa42c(cx: &Cx, fp: &mut Fighter) {
    let (stick, clamp) = ftco_800aa320(fp);
    if is_in_teeter(fp) {
        turn_away_from_ledge(fp);
        return;
    }
    if !((fp.facing_dir * (fp.cpu.x54.x - fp.cur_pos.x)) as f64 >= 0.0) {
        turn_around(fp);
        return;
    }
    let dx = fp.cpu.x54.x - fp.cur_pos.x;
    let dy = fp.cpu.x54.y - fp.cur_pos.y;
    // FMA: fma(dx, dx, dy * dy)
    let dist = sqrtf(dx.mul_add(dx, dy * dy));
    let near = fp.cpu.x38;
    if dist - near > fp.cpu.x3C {
        cmd1(fp, CpuCmd_SetLstickY, 0);
        if b(fp.cpu.xF8_b0()) {
            let m = fp.motion_id;
            let is_run = m == ftCo_MS_Dash as i32 || m == ftCo_MS_Run as i32 || m == ftCo_MS_TurnRun as i32;
            if is_run {
                cmd1(fp, CpuCmd_LstickXTowardDestination, 0x7F);
            } else {
                cmd1(fp, CpuCmd_LstickXTowardDestination, 0);
                cmd1(fp, CpuCmd_WaitFor, 2);
                cmd1(fp, CpuCmd_LstickXTowardDestination, 0x7F);
            }
        } else {
            cmd2(fp, CpuCmd_LstickXTowardDestinationClamped, stick, clamp);
        }
    } else if dist > near {
        cmd1(fp, CpuCmd_SetLstickY, 0);
        let tmp = aa42c_inline0(cx, clamp as f32, dist, fp.cpu.x38, fp.cpu.x3C);
        cmd2(fp, CpuCmd_LstickXTowardDestinationClamped, stick, tmp);
    } else {
        neutral_stick(fp);
    }
    done(fp);
}

/// `ftCo_800AB224`: behaviour 1 on the ground: walk, jump, or turn toward the destination.
fn ftco_800ab224(cx: &mut Cx, fp: &mut Fighter) -> R {
    if b(fp.cpu.xFA_b6()) {
        return Err("ftCo_800AB224 (xFA_b6: under a ceiling) not ported".into());
    }
    if b(fp.cpu.xF9_b1()) {
        let o = fp.cpu.x44;
        if !o.is_null() {
            let ot = unsafe { &*o };
            if ftco_800a2c08(fp) && ftco_800a2170(cx, fp, ot) && ((ot.cur_pos.x - fp.cur_pos.x).abs() as f64) < 37.5 {
                ftco_800a0148(cx, fp);
                return Ok(());
            }
        }
        // 'dodge toward the target' roll: Randf only when the first conditions hold.
        if b(fp.cpu.xF8_b6()) && fp.ground_or_air == GA_Ground as _ {
            let r = cx.rand.randf();
            if r < 0.05f32 * fp.cpu.level as f32 {
                fp.cpu.set_xF8_b6(0);
                return Err("ftCo_800BA2E8 (grounded attack on the target) not ported".into());
            }
        }
    }
    // block_49
    let isl = island_of(cx, fp);
    let dest_on_island = if isl == -1 {
        false
    } else {
        let v = WVec3 { x: fp.cpu.x54.x, y: dadd(5.0, fp.cpu.x54.y), z: 0.0 };
        cx.ground.mp_island_8005ac14(v, -10.0) == isl
    };
    if dest_on_island {
        ftco_800aa42c(cx, fp);
        return Ok(());
    }
    let var_f2 = (fp.cpu.x54.x - fp.cur_pos.x).abs();
    let temp_f1 = fp.cpu.x54.y - fp.cur_pos.y;
    let angle = if var_f2 < 0.00001 && var_f2 > -0.00001 {
        if temp_f1 as f64 > 0.0 { core::f64::consts::FRAC_PI_2 as f32 } else { -(core::f64::consts::FRAC_PI_2 as f32) }
    } else {
        cs::ext::lb_8000D008(temp_f1, var_f2)
    };
    if angle as f64 > 0.6108652334660292 {
        let (cx0, cy0) = (fp.coll_data.cur_pos.x, fp.coll_data.cur_pos.y);
        let blocked = cx.ground.mp_check_ceiling(cx0, cy0, cx0, fp.cpu.x54.y, -1, -1).is_some();
        if blocked {
            if ftco_800a28d0(cx, fp, 1.0) {
                turn_around(fp);
            } else if ftco_800aaf48(cx, fp)? {
                ftco_800a0148(cx, fp);
            } else {
                ftco_800aabc8(fp);
            }
        } else if cx.grkind == Gr_Kind_Icemt as i32 {
            ftco_800a0148(cx, fp);
        } else if !ftco_800a2bd4(fp) {
            if (angle as f64) < 1.3089969288557768 {
                turn_around(fp);
            } else {
                ftco_800aabc8(fp);
            }
        } else {
            ftco_800a0148(cx, fp);
        }
    } else if angle as f64 > -0.7853981573134661 {
        if !ftco_800a2bd4(fp) {
            turn_around(fp);
        } else {
            ftco_800aa844(cx, fp)?;
        }
    } else {
        ftco_800aacd0(cx, fp)?;
    }
    Ok(())
}
