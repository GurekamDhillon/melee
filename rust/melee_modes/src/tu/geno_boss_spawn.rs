//! Geno v5.2 actor projectiles: spawn records and the per-define weapon pool (lane B, docs/specs/geno-v5.2-boss.md §8
//! "spawned projectiles", notes `docs/rust-port/notes/geno-boss.md` "Projectiles").
//!
//! A *record* is the enemy engine's weapon spawn record (Brawl's 0x7C-byte block). The model follows the code:
//! the spawner draws the launch angle and the speed from the record's ranges at the owner's L,
//! v = (cos a * speed * facing, sin a * speed), accel = (cos a, sin a) * the record's accel with facing on x; the
//! weapon init adds the record's y accel and takes its life end; each frame
//! pos += v, then v += accel. Boxes: the record's start script (`fly`) and, for exploding weapons, the explosion
//! of its proc set: velocity zeroed, boxes live for the set's frames, then the weapon is gone. Code addresses and
//! record field positions: the local engine-ops.md.
//! A projectile keeps the owner's L (Rayquaza: "spawned weapons inherit the owner's L").
//!
//! `Op::Spawn` puts the projectile in a free slot of the fixed pool `State::spawn` and emits `EV_SPAWN`; the host
//! steps the pool once a frame with [`tick`], giving the players' hurt circles. A hit emits `EV_SPAWN_HIT` (the host
//! applies the box); removal is owner-side only (`EV_SPAWN_END`).
//!
//! Determinism: plain data in `geno_boss::S`, f32 without fused multiply-add, MSL sin/cos (`melee_runtime::gekko`),
//! no allocation. The pool is hashed only while a slot is live, so an actor without projectiles hashes as before.

use crate::tu::geno_boss::{Define, Num, State, EV_SPAWN, EV_SPAWN_END, EV_SPAWN_HIT, MAX_EVENTS};
use melee_runtime::gekko::{cosf, sinf};

pub const MAX_SPAWN: usize = crate::tu::geno_boss::MAX_SPAWNS;
pub const MAX_BOXES: usize = 2;

/// One live projectile (rec 0xFF = free).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct SpawnSlot {
    pub rec: u8,
    /// players already hit by the current phase's boxes (one hit per player per phase)
    pub hit: u8,
    pub joint: i16,
    pub age: i32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// the owner's L at spawn (the weapon's boxes use it)
    pub l: f32,
    pub ax: f32,
    pub ay: f32,
    /// 0 flying, 1 exploding (age counts the explosion's frames)
    pub phase: u8,
    /// the frame it spawned (hit groups key on it)
    pub born: u32,
}

pub const FREE: SpawnSlot = SpawnSlot { rec: 0xFF, hit: 0, joint: -1, age: 0, x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, l: 0.0, ax: 0.0, ay: 0.0, phase: 0, born: 0 };

/// One box (0619 in the weapon's scripts).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpawnBox {
    pub damage: Num,
    pub angle: i32,
    pub kbg: Num,
    pub bkb: Num,
    /// set knockback (0 = none)
    pub skb: Num,
    pub size: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpawnRecord {
    pub raw_id: u32,
    /// at the owner's position (record mode 2) instead of a bone
    pub at_owner: bool,
    pub life: i32,
    pub scale: f32,
    pub bone: i16,
    /// speed range and launch-angle range (degrees above the facing direction), drawn per spawn
    pub speed: [Num; 2],
    pub angle: [Num; 2],
    /// accel along the launch direction, accel added to y
    pub accel: [f32; 2],
    pub offset: [f32; 2],
    /// boxes while flying (the record's start script)
    pub fly: Vec<SpawnBox>,
    /// explosion: frames and boxes (proc set); None = the weapon just vanishes
    pub explode: Option<(i32, Vec<SpawnBox>)>,
    /// ends (explodes) on the floor (TouchGround proc) / on touching a target (hit flag / HitShell)
    pub end_floor: bool,
    pub end_contact: bool,
    /// homing: search range and turn per frame (radians)
    pub homing: Option<(f32, f32)>,
    /// frames before the homing steers (the spark's lock-on; 0: from the start)
    pub lock: i32,
    /// a floor walker (Porky's Mini-Porky robot): once on the floor it runs at this speed toward the nearest target,
    /// falling by the gravity while airborne: (speed, gravity)
    pub walk: Option<(f32, f32)>,
}

/// Contact radius of a weapon with no fly box (the 0610 shape in its start script is not decoded).
pub const CONTACT_R: f32 = 4.0;

fn lin(n: Num, l: f32) -> f32 {
    match n {
        Num::K(k) => k,
        Num::Lin(b, p) => b + p * l,
        Num::Var(_) => 0.0,
    }
}

fn push(s: &mut State, kind: u8, v: i32) {
    if (s.ev_n as usize) < MAX_EVENTS {
        s.ev[s.ev_n as usize] = (kind, v);
        s.ev_n += 1;
    }
}

/// `Op::Spawn`: a free slot gets the projectile (pool full: nothing, as the native pool). `r1` / `r2` are the
/// actor's two draws (speed, angle), `l` its level. Returns the slot.
pub fn spawn(d: &Define, s: &mut State, record: u8, joint: i16, l: f32, r1: f32, r2: f32) -> Option<usize> {
    let face = if s.aim < 0.0 { -1.0 } else { 1.0 };
    let (x, y) = (s.x, 0.0);
    spawn_at(&d.spawns, s, record, joint, x, y, face, l, r1, r2)
}

/// The same from any owner: its point (x, y), facing and L; `recs` is the owner's record table (a define's, or the
/// VM boss's from [`vm_records`]).
#[allow(clippy::too_many_arguments)]
pub fn spawn_at(recs: &[SpawnRecord], s: &mut State, record: u8, joint: i16, ox: f32, oy: f32, face: f32, l: f32, r1: f32, r2: f32) -> Option<usize> {
    let rc = recs.get(record as usize)?;
    let i = s.spawn.iter().position(|p| p.rec == 0xFF)?;
    let (s0, s1) = (lin(rc.speed[0], l), lin(rc.speed[1], l));
    let (a0, a1) = (lin(rc.angle[0], l), lin(rc.angle[1], l));
    let a = (a0 + (a1 - a0) * r2) * (core::f32::consts::PI / 180.0);
    let v = s0 + (s1 - s0) * r1;
    let (c, sn) = (cosf(a), sinf(a));
    s.spawn[i] = SpawnSlot {
        rec: record,
        hit: 0,
        joint: if joint >= 0 { joint } else { rc.bone },
        age: 0,
        x: if rc.at_owner { ox } else { ox + rc.offset[0] * face },
        y: oy + rc.offset[1],
        vx: c * v * face,
        vy: sn * v,
        l,
        ax: c * rc.accel[0] * face,
        ay: sn * rc.accel[0] + rc.accel[1],
        phase: 0,
        born: s.frame.wrapping_add(s.vm.tick),
    };
    if crate::tu::geno_boss::log_on() {
        eprintln!("geno_boss: f{} spawn record {} slot {} joint {} v {:.2} a {:.1}", s.frame.wrapping_add(s.vm.tick), rc.raw_id, i, s.spawn[i].joint, v, a * 180.0 / core::f32::consts::PI);
    }
    push(s, EV_SPAWN, i as i32);
    Some(i)
}

/// The host's placement of a fresh projectile at its bone (world x, y), on the frame of `EV_SPAWN`.
pub fn place(slot: usize, x: f32, y: f32) {
    unsafe {
        if let Some(p) = crate::tu::geno_boss::S.spawn.get_mut(slot) {
            if p.rec != 0xFF {
                p.x = x;
                p.y = y;
            }
        }
    }
}

/// A target's hurt circle: x, y, radius (None = absent).
pub type Target = Option<(f32, f32, f32)>;
/// A hit for the host: slot, target, damage, angle, kbg, bkb, set knockback.
pub type Hit = (u8, u8, f32, i32, f32, f32, f32);

fn touches(x: f32, y: f32, r: f32, t: &Target) -> bool {
    match t {
        Some((tx, ty, tr)) => {
            let (dx, dy) = (x - tx, y - ty);
            let rr = r + tr;
            dx * dx + dy * dy <= rr * rr
        }
        None => false,
    }
}

/// Turn (vx, vy) toward (dx, dy) by at most `turn` radians, keeping the speed.
fn steer(vx: f32, vy: f32, dx: f32, dy: f32, turn: f32) -> (f32, f32) {
    let cross = vx * dy - vy * dx;
    let dot = vx * dx + vy * dy;
    if cross == 0.0 && dot >= 0.0 {
        return (vx, vy);
    }
    let t = if cross >= 0.0 { turn } else { -turn };
    let (c, s) = (cosf(t), sinf(t));
    (vx * c - vy * s, vx * s + vy * c)
}

/// Step the pool one frame (after `geno_boss::tick`). Returns the number of hits written to `out`.
pub fn tick(d: &Define, targets: &[Target; 4], out: &mut [Hit; MAX_SPAWN]) -> usize {
    if !unsafe { crate::tu::geno_boss::S.live } {
        return 0;
    }
    step(&d.spawns, targets, out)
}

/// One pool frame against a record table (the define's, or a VM boss's).
pub fn step(recs: &[SpawnRecord], targets: &[Target; 4], out: &mut [Hit; MAX_SPAWN]) -> usize {
    let s = unsafe { &mut crate::tu::geno_boss::S };
    let mut n = 0;
    for i in 0..MAX_SPAWN {
        let p = s.spawn[i];
        if p.rec == 0xFF {
            continue;
        }
        let Some(rc) = recs.get(p.rec as usize) else {
            s.spawn[i] = FREE;
            continue;
        };
        let mut q = p;
        q.age += 1;
        let mut end = false;
        let mut boom = false;
        if q.phase == 0 {
            if let Some((range, turn)) = rc.homing.filter(|_| q.age > rc.lock) {
                // the nearest target in range
                let mut best: Option<(f32, f32, f32)> = None;
                for t in targets.iter().flatten() {
                    let (dx, dy) = (t.0 - q.x, t.1 - q.y);
                    let d2 = dx * dx + dy * dy;
                    if d2 <= range * range && best.is_none_or(|b| d2 < b.2) {
                        best = Some((dx, dy, d2));
                    }
                }
                if let Some((dx, dy, _)) = best {
                    (q.vx, q.vy) = steer(q.vx, q.vy, dx, dy, turn);
                }
            }
            if let Some((speed, grav)) = rc.walk {
                // a floor walker: on the floor (y 0, the boss arenas' floor line) it runs at its speed toward the
                // nearest target; in the air it falls
                if q.y <= 0.0 && q.vy <= 0.0 {
                    q.y = 0.0;
                    q.vy = 0.0;
                    q.ay = 0.0;
                    q.ax = 0.0;
                    let mut best: Option<f32> = None;
                    for t in targets.iter().flatten() {
                        let dx = t.0 - q.x;
                        let ab = |v: f32| if v < 0.0 { -v } else { v };
                        if best.is_none_or(|b| ab(dx) < ab(b)) {
                            best = Some(dx);
                        }
                    }
                    q.vx = match best {
                        Some(dx) if dx < 0.0 => -speed,
                        Some(_) => speed,
                        None => if q.vx < 0.0 { -speed } else { speed },
                    };
                } else if grav > 0.0 {
                    q.ay = -grav;
                }
            }
            q.x += q.vx;
            q.y += q.vy;
            q.vx += q.ax;
            q.vy += q.ay;
            if rc.walk.is_some() && q.y < 0.0 {
                q.y = 0.0;
                q.vy = 0.0;
            }
            let floor = rc.end_floor && q.vy < 0.0 && q.y <= 0.0;
            let mut contact = false;
            for (t, tg) in targets.iter().enumerate() {
                if tg.is_none() || crate::tu::geno_boss::bit(q.hit, t as i32) {
                    continue;
                }
                for b in rc.fly.iter().take(MAX_BOXES) {
                    if touches(q.x, q.y, b.size * rc.scale, tg) {
                        q.hit |= crate::tu::geno_boss::m8(t as i32);
                        if n < MAX_SPAWN {
                            out[n] = (i as u8, t as u8, lin(b.damage, q.l), b.angle, lin(b.kbg, q.l), lin(b.bkb, q.l), lin(b.skb, q.l));
                            n += 1;
                        }
                        push(s, EV_SPAWN_HIT, (i as i32) << 8 | t as i32);
                        contact = true;
                        break;
                    }
                }
                if rc.fly.is_empty() && rc.end_contact && touches(q.x, q.y, CONTACT_R * rc.scale, tg) {
                    contact = true;
                }
            }
            if floor || (contact && rc.end_contact) {
                boom = rc.explode.is_some();
                end = !boom;
            }
            if q.age >= rc.life.max(1) {
                end = true;
                boom = false;
            }
            if boom {
                q.phase = 1;
                q.age = 0;
                q.hit = 0;
                q.vx = 0.0;
                q.vy = 0.0;
                q.ax = 0.0;
                q.ay = 0.0;
                if crate::tu::geno_boss::log_on() {
                    eprintln!("geno_boss: f{} spawn slot {} explodes at ({:.1}, {:.1})", s.frame.wrapping_add(s.vm.tick), i, q.x, q.y);
                }
            }
        }
        if q.phase == 1 && !end {
            let (frames, boxes) = rc.explode.as_ref().map(|(f, b)| (*f, &b[..])).unwrap_or((0, &[][..]));
            for (t, tg) in targets.iter().enumerate() {
                if tg.is_none() || crate::tu::geno_boss::bit(q.hit, t as i32) {
                    continue;
                }
                if let Some(b) = boxes.iter().take(MAX_BOXES).find(|b| touches(q.x, q.y, b.size * rc.scale, tg)) {
                    q.hit |= crate::tu::geno_boss::m8(t as i32);
                    if n < MAX_SPAWN {
                        out[n] = (i as u8, t as u8, lin(b.damage, q.l), b.angle, lin(b.kbg, q.l), lin(b.bkb, q.l), lin(b.skb, q.l));
                        n += 1;
                    }
                    push(s, EV_SPAWN_HIT, (i as i32) << 8 | t as i32);
                }
            }
            if q.age >= frames.max(1) {
                end = true;
            }
        }
        if end {
            if crate::tu::geno_boss::log_on() {
                eprintln!("geno_boss: f{} spawn slot {} removed at age {} ({:.1}, {:.1})", s.frame.wrapping_add(s.vm.tick), i, q.age, q.x, q.y);
            }
            s.spawn[i] = FREE;
            push(s, EV_SPAWN_END, i as i32);
        } else {
            s.spawn[i] = q;
        }
    }
    n
}

/// Owner-side removal of every projectile (a defeat, a message that clears the pool).
pub fn clear(s: &mut State) {
    for i in 0..MAX_SPAWN {
        if s.spawn[i].rec != 0xFF {
            s.spawn[i] = FREE;
            push(s, EV_SPAWN_END, i as i32);
        }
    }
}

pub fn live_count() -> usize {
    unsafe { crate::tu::geno_boss::S.spawn.iter().filter(|p| p.rec != 0xFF).count() }
}

/// The pool's words for `state_hash` (only while a slot is live).
pub fn hash_into(s: &State, mix: &mut dyn FnMut(u32)) {
    if s.spawn.iter().all(|p| p.rec == 0xFF) {
        return;
    }
    for p in &s.spawn {
        mix(p.rec as u32 | (p.hit as u32) << 8 | (p.joint as u16 as u32) << 16);
        mix(p.age as u32 | (p.phase as u32) << 24);
        mix(p.born);
        for v in [p.x, p.y, p.vx, p.vy, p.l, p.ax, p.ay] {
            mix(v.to_bits());
        }
    }
}

// ------------------------------------------------------------------------------------------------- VM bosses

/// A VM-run boss's weapon records, from the owner's local IR (`<ir dir>/<name>.brawl.ir.json` or
/// `enemies/<name>.brawl.ir.json`, its `weapon_records`; never in the repo). Loaded once per boss on a helper thread,
/// outside the frame loop; empty when the file is absent (the host then spawns nothing).
pub fn vm_records(name: &str) -> &'static [SpawnRecord] {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static M: OnceLock<Mutex<HashMap<String, &'static [SpawnRecord]>>> = OnceLock::new();
    let m = M.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(v) = m.lock().unwrap().get(name) {
        return v;
    }
    let n = name.to_string();
    let recs = std::thread::spawn(move || {
        let dir = crate::tu::geno_boss_ir::ir_dir()?;
        let f = format!("{n}.brawl.ir.json");
        let text = std::fs::read_to_string(dir.join(&f)).or_else(|_| std::fs::read_to_string(dir.join("enemies").join(&f))).ok()?;
        let j = crate::tu::geno_boss_ir::parse_json(&text).ok()?;
        Some(lower_records(j.get("boss")))
    })
    .join()
    .ok()
    .flatten()
    .unwrap_or_default();
    eprintln!("geno_boss: {name}: {} weapon records", recs.len());
    let v: &'static [SpawnRecord] = Box::leak(recs.into_boxed_slice());
    m.lock().unwrap().insert(name.to_string(), v);
    v
}

/// The record index for a 650E's param id (None: not in the table).
pub fn record_index(recs: &[SpawnRecord], raw_id: u32) -> Option<u8> {
    recs.iter().position(|r| r.raw_id == raw_id).map(|i| i as u8)
}

/// A live projectile's boxes this frame, for the host's hit pipeline: (slot, x, y, box, owner L, hit group).
/// The group changes per projectile and per phase, so the explosion re-arms after a flying hit.
pub fn live_boxes(recs: &[SpawnRecord], mut f: impl FnMut(usize, f32, f32, &SpawnBox, f32, u32)) {
    let s = unsafe { &crate::tu::geno_boss::S };
    for (i, p) in s.spawn.iter().enumerate() {
        let Some(rc) = recs.get(p.rec as usize).filter(|_| p.rec != 0xFF) else { continue };
        let boxes: &[SpawnBox] = if p.phase == 0 { &rc.fly } else { rc.explode.as_ref().map(|e| &e.1[..]).unwrap_or(&[]) };
        let grp = 0x4000_0000 | (p.born & 0xFFFF) << 8 | (i as u32) << 1 | p.phase as u32;
        for b in boxes.iter().take(MAX_BOXES) {
            f(i, p.x, p.y, b, p.l, grp);
        }
    }
}

/// Box numbers at a projectile's L.
pub fn at_l(b: &SpawnBox, l: f32) -> (f32, f32, f32, f32) {
    (lin(b.damage, l), lin(b.kbg, l), lin(b.bkb, l), lin(b.skb, l))
}

// ------------------------------------------------------------------------------------------------- IR lowering

use crate::tu::geno_boss_ir::J;

fn jnum(j: &J) -> Num {
    match j.num() {
        Some(v) => Num::K(v as f32),
        None => {
            let b = j.get("base").num().unwrap_or(0.0) as f32;
            let p = j.get("per_level").num().unwrap_or(0.0) as f32;
            if p == 0.0 {
                Num::K(b)
            } else {
                Num::Lin(b, p)
            }
        }
    }
}

fn jboxes(a: &J) -> Vec<SpawnBox> {
    a.arr()
        .iter()
        .take(MAX_BOXES)
        .map(|b| SpawnBox {
            damage: jnum(b.get("damage")),
            angle: b.get("angle").num().unwrap_or(0.0) as i32, // semantics: allow(load time: IR lowering on a helper thread, never in the sim)
            kbg: jnum(b.get("knockback_growth")),
            bkb: jnum(b.get("base_knockback")),
            skb: jnum(b.get("set_knockback")),
            size: b.get("size").num().unwrap_or(0.0) as f32,
        })
        .collect()
}

/// `boss.weapon_records` (local IR) -> records; the index in the result is the `Op::Spawn` record index.
pub fn lower_records(boss: &J) -> Vec<SpawnRecord> {
    let mut v = vec![];
    for r in boss.get("weapon_records").arr() {
        let a = |k: &str, i: usize| r.get(k).arr().get(i).map(jnum).unwrap_or(Num::K(0.0));
        let f = |k: &str, i: usize| r.get(k).arr().get(i).and_then(|x| x.num()).unwrap_or(0.0) as f32;
        let ends = |w: &str| r.get("on_end").arr().iter().any(|e| e.str() == w);
        let ex = r.get("explosion");
        let hm = r.get("homing");
        v.push(SpawnRecord {
            raw_id: r.get("raw_id").num().unwrap_or(0.0) as u32, // semantics: allow(load time: IR lowering on a helper thread, never in the sim)
            at_owner: r.get("mode").str() == "owner",
            life: r.get("life").num().unwrap_or(60.0) as i32, // semantics: allow(load time: IR lowering on a helper thread, never in the sim)
            scale: r.get("scale").num().unwrap_or(1.0) as f32,
            bone: r.get("bone").num().unwrap_or(-1.0) as i16, // semantics: allow(load time: IR lowering on a helper thread, never in the sim)
            speed: [a("speed", 0), a("speed", 1)],
            angle: [a("angle", 0), a("angle", 1)],
            accel: [f("accel", 0), f("accel", 1)],
            offset: [f("offset", 0), f("offset", 1)],
            fly: jboxes(r.get("fly_boxes")),
            explode: if ex.is_null() { None } else { Some((ex.get("frames").num().unwrap_or(1.0) as i32, jboxes(ex.get("boxes")))) }, // semantics: allow(load time: IR lowering on a helper thread, never in the sim)
            end_floor: ends("floor"),
            end_contact: ends("contact"),
            // turn: the second of the homing set's numbers (0.06 rad for Duon); its role is inferred, not traced
            lock: r.get("lock_frames").num().unwrap_or(0.0) as i32, // semantics: allow(load time: IR lowering on a helper thread, never in the sim)
            walk: {
                let w = r.get("walk");
                if w.is_null() { None } else { Some((w.get("speed").num().unwrap_or(1.0) as f32, w.get("gravity").num().unwrap_or(0.0) as f32)) }
            },
            // a turn given as {base, per_level} (Porky's spark: 2.2 + 0.3 L, unit not traced) reads as degrees at its
            // base [I: radians would turn it at once]
            homing: if hm.is_null() {
                None
            } else {
                let t = hm.get("numbers").arr().get(1);
                let turn = t.and_then(|x| x.num()).map(|v| v as f32).or_else(|| t.and_then(|x| x.get("base").num()).map(|v| (v as f32).to_radians()));
                Some((hm.get("range").num().unwrap_or(0.0) as f32, turn.unwrap_or(0.0)))
            },
        });
    }
    v
}

/// A timeline's spawn op text -> (raw record id, joint): "650E vabf6 0 26" (record 0xABF6 at bone 26), "650E vabee 0"
/// (the record's own bone), "650E param44006". None = not a spawn op.
pub fn parse_spawn_text(text: &str) -> Option<(u32, i16)> {
    let mut w = text.split_whitespace();
    if w.next()? != "650E" {
        return None;
    }
    let r = w.next()?;
    let id = if let Some(h) = r.strip_prefix('v') {
        u32::from_str_radix(h, 16).ok()?
    } else {
        r.strip_prefix("param")?.parse().ok()?
    };
    let _ = w.next();
    let joint = w.next().and_then(|j| j.parse::<i16>().ok()).unwrap_or(-1);
    Some((id, joint))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tu::geno_boss::{self as gb, Op, World, S};

    /// Every motion an IR's timelines name, with its length (stand-in host motions for a boss with no Melee host).
    fn ir_motions(file: &str) -> Option<(J, ())> {
        let dir = crate::tu::geno_boss_ir::ir_dir()?;
        let text = std::fs::read_to_string(dir.join(file)).ok()?;
        // the analyst IRs write unread floats as NaN, which the IR reader does not take
        Some((crate::tu::geno_boss_ir::parse_json(&text.replace("NaN", "null")).ok()?, ()))
    }

    #[test]
    fn parses_spawn_ops() {
        assert_eq!(parse_spawn_text("650E v1234 0 7"), Some((0x1234, 7)));
        assert_eq!(parse_spawn_text("650E vff 0"), Some((0xFF, -1)));
        assert_eq!(parse_spawn_text("650E param99"), Some((99, -1)));
        assert_eq!(parse_spawn_text("0603 0"), None);
    }

    /// Lower a local IR's records and timelines' spawn ops, fire one motion's spawns, fly them into a target and out.
    fn run_ir(file: &str, motion: &str, expect: usize, target: (f32, f32)) {
        let Some((j, ms)) = ir_motions(file) else {
            eprintln!("skipped: no local IR {file}");
            return;
        };
        let recs = lower_records(j.get("boss"));
        assert!(!recs.is_empty(), "{file}: no weapon_records (run gen_weapons.py)");
        // the motion's spawn ops, lowered the way geno_boss_ir lowers a timeline
        let tl = j.get("boss").get("timelines").arr().iter().find(|t| t.get("motion").str() == motion).expect("motion");
        let mut evs = vec![];
        for e in tl.get("events").arr() {
            // a spawn under a script condition ("when", e.g. a level test) is not taken at the test's L
            if !e.get("when").is_null() {
                continue;
            }
            if let Some((id, jt)) = parse_spawn_text(e.get("op").get("text").str()) {
                let ri = recs.iter().position(|r| r.raw_id == id).expect("record lowered") as u8;
                evs.push(gb::TimelineEvent { frame: e.get("frame").num().unwrap() as i32, count: 1, every: 0, op: Op::Spawn { record: ri, joint: jt } });
            }
        }
        assert_eq!(evs.len(), expect, "{file} {motion}: spawn ops");
        let len = tl.get("frames").num().unwrap() as i32;
        let mut d = tiny_define(recs, gb::Motion { name: motion.into(), len, looping: false, timeline: evs });
        d.name = file.into();
        let w = World::default();
        gb::begin(&d, w, 5, 1000, 0x1234);
        let targets: [Target; 4] = [Some((target.0, target.1, 30.0)), None, None, None];
        let mut out = [(0, 0, 0.0, 0, 0.0, 0.0, 0.0); MAX_SPAWN];
        let (mut spawned, mut hits, mut ends) = (0, 0, 0);
        let mut max_live = 0;
        for _ in 0..(len + 700) {
            gb::tick(&d, w);
            let nh = tick(&d, &targets, &mut out);
            hits += nh;
            let s = unsafe { S };
            for &(k, v) in s.ev.iter().take(s.ev_n as usize) {
                if k == EV_SPAWN {
                    place(v as usize, s.x, 30.0); // the host puts it at its bone, 30 above the floor here
                }
                spawned += (k == EV_SPAWN) as usize;
                ends += (k == EV_SPAWN_END) as usize;
            }
            max_live = max_live.max(live_count());
            for h in &out[..nh] {
                assert!(h.2 > 0.0);
            }
        }
        eprintln!("{file} {motion}: spawned {spawned} hits {hits} removed {ends} max live {max_live}");
        assert_eq!(spawned, expect);
        assert_eq!(ends, expect, "every projectile is removed by its owner");
        assert!(hits >= 1, "a projectile reaches the target");
        assert_eq!(live_count(), 0);
        gb::stop();
    }

    fn tiny_define(spawns: Vec<SpawnRecord>, m: gb::Motion) -> Define {
        Define {
            name: "spawn-test".into(),
            parts: vec![gb::Part { name: "core".into(), role: gb::Role::Core, hp: Num::K(100.0), on_damaged: None, thresholds: vec![], on_zero: None, intangible_in: 0, int_level: false, facing: 0.0, coll: None }],
            vars: vec![],
            statuses: vec![gb::Status { index: 0, name: "A".into(), enter: vec![Op::Motion(0)], frame: vec![], frame_once: false, intangible: false }],
            rules: vec![],
            tables: vec![],
            kinetics: vec![],
            motions: vec![m],
            attacks: vec![],
            messages: vec![],
            level_rows: vec![vec![2.5; 15]],
            strength0: 1,
            start_status: 0,
            slot_w: [1.0; 4],
            slot_repeat: 0.3,
            route_mask: 0,
            route_factor: 1.0,
            defeat_status: [None, None],
            goal_snap: true,
            spawn_x: 0.0,
            camera: None,
            window: (-50, 250),
            passes: vec![],
            spawns,
        }
    }

    #[test]
    fn galleom_grenades_and_missiles() {
        // grenades lob forward and fall (accel y < 0), missiles fly level-ish: targets on each path
        run_ir("galleom.brawl.ir.json", "Grenade", 4, (60.0, 0.0));
        run_ir("galleom.brawl.ir.json", "Shoot", 4, (60.0, 20.0));
    }

    #[test]
    fn duon_shots() {
        let Some((j, _)) = ir_motions("duon.brawl.ir.json") else { return };
        // every timeline with a spawn op fires and its shots are all removed
        for t in j.get("boss").get("timelines").arr() {
            let n = t.get("events").arr().iter().filter(|e| e.get("when").is_null() && parse_spawn_text(e.get("op").get("text").str()).is_some()).count();
            if n > 0 && n <= MAX_SPAWN {
                run_ir_any(&j, t.get("motion").str(), n);
            }
        }
    }

    fn run_ir_any(j: &J, motion: &str, n: usize) {
        let recs = lower_records(j.get("boss"));
        let tl = j.get("boss").get("timelines").arr().iter().find(|t| t.get("motion").str() == motion).unwrap();
        let mut evs = vec![];
        for e in tl.get("events").arr() {
            // a spawn under a script condition ("when", e.g. a level test) is not taken at the test's L
            if !e.get("when").is_null() {
                continue;
            }
            if let Some((id, jt)) = parse_spawn_text(e.get("op").get("text").str()) {
                let Some(ri) = recs.iter().position(|r| r.raw_id == id) else {
                    eprintln!("duon {motion}: record {id} not read, op skipped");
                    continue;
                };
                evs.push(gb::TimelineEvent { frame: e.get("frame").num().unwrap() as i32, count: 1, every: 0, op: Op::Spawn { record: ri as u8, joint: jt } });
            }
        }
        let k = evs.len();
        let len = tl.get("frames").num().unwrap() as i32;
        let d = tiny_define(recs, gb::Motion { name: motion.into(), len, looping: false, timeline: evs });
        gb::begin(&d, World::default(), 5, 1000, 9);
        let mut out = [(0, 0, 0.0, 0, 0.0, 0.0, 0.0); MAX_SPAWN];
        let (mut sp, mut en) = (0, 0);
        for _ in 0..(len + 700) {
            gb::tick(&d, World::default());
            tick(&d, &[None; 4], &mut out);
            let s = unsafe { S };
            for &(e, v) in s.ev.iter().take(s.ev_n as usize) {
                if e == EV_SPAWN {
                    place(v as usize, s.x, 30.0);
                }
                sp += (e == EV_SPAWN) as usize;
                en += (e == EV_SPAWN_END) as usize;
            }
        }
        eprintln!("duon {motion}: {n} spawn ops, {k} lowered, spawned {sp}, removed {en}");
        assert_eq!(sp, k);
        assert_eq!(en, k);
        gb::stop();
    }

    #[test]
    fn hammer_bro_hammer() {
        run_ir("enemies/hammer-bro.brawl.ir.json", "Hammer", 1, (25.0, 20.0));
    }

    /// Rollback: snapshot the state mid-flight, run on, restore, rerun: same hashes (the synctest property, headless).
    #[test]
    fn pool_rolls_back() {
        let Some((j, _)) = ir_motions("galleom.brawl.ir.json") else { return };
        let recs = lower_records(j.get("boss"));
        let evs = vec![gb::TimelineEvent { frame: 2, count: 4, every: 5, op: Op::Spawn { record: 1, joint: 26 } }];
        let d = tiny_define(recs, gb::Motion { name: "Shoot".into(), len: 200, looping: true, timeline: evs });
        gb::begin(&d, World::default(), 5, 1000, 77);
        let tg: [Target; 4] = [Some((30.0, -5.0, 6.0)), None, None, None];
        let mut out = [(0, 0, 0.0, 0, 0.0, 0.0, 0.0); MAX_SPAWN];
        let mut step = || {
            gb::tick(&d, World::default());
            tick(&d, &tg, &mut out);
            gb::state_hash().unwrap()
        };
        for _ in 0..10 {
            step();
        }
        let snap = unsafe { S };
        let a: Vec<u32> = (0..60).map(|_| step()).collect();
        unsafe { S = snap };
        let b: Vec<u32> = (0..60).map(|_| step()).collect();
        assert_eq!(a, b);
        // the pool is in the hash while live
        unsafe { S = snap };
        let h0 = gb::state_hash().unwrap();
        let live = unsafe { S.spawn.iter().position(|p| p.rec != 0xFF) }.expect("a live projectile at the snapshot");
        unsafe { S.spawn[live].x += 1.0 };
        assert_ne!(h0, gb::state_hash().unwrap());
        gb::stop();
    }
}
