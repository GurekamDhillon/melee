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
    let m = fp.motion_id;
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
        return Err("ftCo_CpuTargetAct (a valid target) not ported".into());
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

/// `ftCo_800A53DC`: the fighter to attack (lowest rank, then nearest).
fn ftco_800a53dc(cx: &Cx, me: *mut Fighter) -> R<*mut Fighter> {
    let fp = unsafe { &*me };
    // Only a valid enemy changes the answer; with none, NULL.
    for &cur in &cx.fps {
        if cur == me {
            continue;
        }
        let o = unsafe { &*cur };
        if cx.out_of_bounds(&fp.cpu, o.cur_pos.x, o.cur_pos.y) || ftco_is_ally(fp, o) || ftco_800a1c44(o) || inline_d1(o)? {
            continue;
        }
        return Err("ftCo_800A53DC (a valid attack target) not ported".into());
    }
    Ok(core::ptr::null_mut())
}

/// `ftCo_800A75DC(fp, target)`: head for the target (NULL: nothing).
fn ftco_800a75dc(_cx: &Cx, _me: *mut Fighter, target: *mut Fighter) -> R {
    if target.is_null() {
        return Ok(());
    }
    Err("ftCo_800A75DC (head for a target) not ported".into())
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
        return Err("ftCo_800ADE48: x221A_b3 (just hit) path not ported".into());
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
    if ftco_800b8a9c(fp)? {
        cs::ftco_800b4a78(fp);
        fp.cpu.x18 = 2;
        return Ok(());
    }
    if fp.cpu.x18 == 8 {
        return Ok(());
    }
    let r = ftco_800b732c(fp)?;
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

/// `ftCo_800B9CBC`: decide on a ranged attack at the target.
fn ftco_800b9cbc(_cx: &mut Cx, fp: &mut Fighter) -> R<bool> {
    fp.cpu.xA4 = 0;
    if fp.cpu.x44.is_null() {
        return Ok(false);
    }
    Err("ftCo_800B9CBC (a target) not ported".into())
}

/// `ftCo_800B8A9C`: decide on a melee attack at the target.
fn ftco_800b8a9c(fp: &mut Fighter) -> R<bool> {
    if !b(fp.cpu.xF9_b2()) {
        return Ok(false);
    }
    fp.cpu.xEC = 0;
    fp.cpu.xC8 = 0;
    // target == NULL (or Pass, an untargetable target, ...): no attack.
    if fp.cpu.x44.is_null() || fp.motion_id == ftCo_MS_Pass as i32 || ftco_800a1ca8(fp) {
        fp.cpu.xA4 = 0;
        return Ok(false);
    }
    Err("ftCo_800B8A9C (a target) not ported".into())
}

/// `ftCo_800B732C`: grab / ledge decisions against the target.
fn ftco_800b732c(fp: &Fighter) -> R<i32> {
    if fp.cpu.x44.is_null() || fp.ground_or_air == GA_Air as _ {
        return Ok(0);
    }
    Err("ftCo_800B732C (a target) not ported".into())
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
        c => return Err(format!("ftCo_800B2790: behaviour {c} not ported")),
    }
    cs::ftco_800b49f4(fp);
    Ok(())
}

/// `ftCo_800ADC28`: keep charging specials (Donkey Kong, Samus, Mewtwo, Kirby).
fn ftco_800adc28(fp: &Fighter) -> R {
    if fp.cpu.x18 == 0xA {
        return Ok(());
    }
    let k = fp.kind;
    if k == Ft_Kind_Donkey || k == Ft_Kind_Samus || k == Ft_Kind_Mewtwo || k == Ft_Kind_Kirby {
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
    if fp.motion_id == ftCo_MS_RebirthWait as i32 {
        return Err("ftCo_800A08F0 (RebirthWait) not ported".into());
    }
    if fp.ground_or_air == GA_Air as _ {
        return ftco_800a9cb4(cx, fp);
    }
    Err("ftCo_800AB224 (grounded approach) not ported".into())
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
        (fp.cur_pos.y as f64 + ((fp.pos_delta.y * x_time) as f64 - 0.5f64 * (gravity * s) as f64)) as f32
    } else {
        let s = sqrtf(terminal_time);
        let xd = x_time - terminal_time;
        (fp.cur_pos.y as f64 + (((fp.pos_delta.y * terminal_time) as f64 - 0.5f64 * (gravity * s) as f64) - (xd * fp.co_attrs.terminal_velocity) as f64)) as f32
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
        let k = o.kind;
        if k == Ft_Kind_GKoops || k == Ft_Kind_Koopa || k == Ft_Kind_Yoshi || k == Ft_Kind_Samus {
            return Err("ftCo_800BB768 (Bowser / Yoshi / Samus threat) not ported".into());
        }
    }
    for it in cx.items {
        use super::fox::ItemClass as C;
        // it_8026C1B4: the item has an enabled hitbox.
        if !matches!(it.class, C::GreensBlock) {
            return Err("ftCo_800BB220 (an item with a hitbox) not ported".into());
        }
    }
    Ok(())
}
