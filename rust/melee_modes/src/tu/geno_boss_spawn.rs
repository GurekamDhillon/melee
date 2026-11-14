//! Geno v5.2 actor projectiles: spawn records and the per-define weapon pool (lane B, docs/specs/geno-v5.2-boss.md §8
//! "spawned projectiles", notes `docs/rust-port/notes/geno-boss.md` "Projectiles").
//!
//! A *record* is the enemy engine's weapon spawn record (Brawl's 0x7C-byte block read by the spawn helper chain:
//! mode, life, scale, default bone, speed range, launch-angle range, per-frame acceleration, offset). `Op::Spawn`
//! draws the speed and angle from the actor's LCG at the actor's L (the weapon keeps that L: Rayquaza's "spawned
//! weapons inherit the owner's L"), puts the projectile in a free slot of the fixed pool `State::spawn`, and emits
//! `EV_SPAWN`. The host steps the pool once a frame with [`tick`], giving the players' hurt circles; a hit emits
//! `EV_SPAWN_HIT` (the host applies the record's box) and frees the slot; so does life running out and, for a
//! falling record, the floor. Removal is always owner-side: nothing outside the actor's state frees a slot.
//!
//! Determinism: plain data in `geno_boss::S`, f32 without fused multiply-add, MSL sin/cos (`melee_runtime::gekko`),
//! no allocation. The pool is hashed only while a slot is live, so an actor without projectiles hashes as before.

use crate::tu::geno_boss::{Define, Num, State, EV_SPAWN, EV_SPAWN_END, EV_SPAWN_HIT, MAX_EVENTS};

pub const MAX_SPAWN: usize = 8;

/// One live projectile (rec 0xFF = free).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct SpawnSlot {
    pub rec: u8,
    /// players already hit (one hit per player per projectile)
    pub hit: u8,
    pub joint: i16,
    pub age: i32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// the owner's L at spawn (the weapon's boxes use it)
    pub l: f32,
}

pub const FREE: SpawnSlot = SpawnSlot { rec: 0xFF, hit: 0, joint: -1, age: 0, x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, l: 0.0 };

/// The box a projectile carries. Brawl keeps it in the weapon's own command set (the record's sub-record), which is
/// not read yet: records lowered without one get [`SpawnBox::UNREAD`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpawnBox {
    pub damage: Num,
    pub angle: i32,
    pub kbg: Num,
    pub bkb: Num,
    /// radius at scale 1
    pub size: f32,
}

impl SpawnBox {
    pub const UNREAD: SpawnBox = SpawnBox { damage: Num::K(8.0), angle: 45, kbg: Num::K(60.0), bkb: Num::K(30.0), size: 4.0 };
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
    /// per-frame acceleration: x along the facing, y up
    pub accel: [f32; 2],
    pub offset: [f32; 2],
    pub hitbox: SpawnBox,
}

impl SpawnRecord {
    /// A falling record ends on the floor (y <= 0 while descending): the weapon's TouchGround proc.
    pub fn falls(&self) -> bool {
        self.accel[1] < 0.0
    }
}

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
    let rc = d.spawns.get(record as usize)?;
    let i = s.spawn.iter().position(|p| p.rec == 0xFF)?;
    let (s0, s1) = (lin(rc.speed[0], l), lin(rc.speed[1], l));
    let (a0, a1) = (lin(rc.angle[0], l), lin(rc.angle[1], l));
    let v = s0 + (s1 - s0) * r1;
    let a = (a0 + (a1 - a0) * r2) * (core::f32::consts::PI / 180.0);
    let face = if s.aim < 0.0 { -1.0 } else { 1.0 };
    s.spawn[i] = SpawnSlot {
        rec: record,
        hit: 0,
        joint: if joint >= 0 { joint } else { rc.bone },
        age: 0,
        x: s.x + rc.offset[0] * face,
        y: rc.offset[1],
        vx: melee_runtime::gekko::cosf(a) * v * face,
        vy: melee_runtime::gekko::sinf(a) * v,
        l,
    };
    if crate::tu::geno_boss::log_on() {
        eprintln!("geno_boss: f{} spawn record {} slot {} joint {} v {:.2} a {:.1}", s.frame, rc.raw_id, i, s.spawn[i].joint, v, a * 180.0 / core::f32::consts::PI);
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

/// Step the pool one frame (after `geno_boss::tick`): move, test against the targets, end on life / floor. Returns
/// the hits as (slot, target, damage, angle, kbg, bkb) for the host; the same go out as `EV_SPAWN_HIT`
/// (slot << 8 | target), removals as `EV_SPAWN_END`.
pub fn tick(d: &Define, targets: &[Target; 4], out: &mut [(u8, u8, f32, i32, f32, f32); MAX_SPAWN]) -> usize {
    let s = unsafe { &mut crate::tu::geno_boss::S };
    if !s.live {
        return 0;
    }
    let mut n = 0;
    let face = if s.aim < 0.0 { -1.0 } else { 1.0 };
    for i in 0..MAX_SPAWN {
        let p = s.spawn[i];
        if p.rec == 0xFF {
            continue;
        }
        let Some(rc) = d.spawns.get(p.rec as usize) else {
            s.spawn[i] = FREE;
            continue;
        };
        let mut q = p;
        q.age += 1;
        q.vx += rc.accel[0] * if p.vx < 0.0 { -1.0 } else if p.vx > 0.0 { 1.0 } else { face };
        q.vy += rc.accel[1];
        q.x += q.vx;
        q.y += q.vy;
        let r = rc.hitbox.size * rc.scale;
        let mut end = q.age >= rc.life.max(1) || (rc.falls() && q.vy < 0.0 && q.y <= 0.0);
        for (t, tg) in targets.iter().enumerate() {
            let Some((tx, ty, tr)) = tg else { continue };
            if q.hit >> t & 1 != 0 {
                continue;
            }
            let (dx, dy) = (q.x - tx, q.y - ty);
            let rr = r + tr;
            if dx * dx + dy * dy <= rr * rr {
                q.hit |= 1 << t;
                let b = rc.hitbox;
                if n < MAX_SPAWN {
                    out[n] = (i as u8, t as u8, lin(b.damage, q.l), b.angle, lin(b.kbg, q.l), lin(b.bkb, q.l));
                    n += 1;
                }
                push(s, EV_SPAWN_HIT, (i as i32) << 8 | t as i32);
                if crate::tu::geno_boss::log_on() {
                    eprintln!("geno_boss: f{} spawn slot {} hits target {} at ({:.1}, {:.1})", s.frame, i, t, q.x, q.y);
                }
                end = true;
                break;
            }
        }
        if end {
            if crate::tu::geno_boss::log_on() {
                eprintln!("geno_boss: f{} spawn slot {} removed at age {} ({:.1}, {:.1})", s.frame, i, q.age, q.x, q.y);
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
        mix(p.age as u32);
        mix(p.x.to_bits());
        mix(p.y.to_bits());
        mix(p.vx.to_bits());
        mix(p.vy.to_bits());
        mix(p.l.to_bits());
    }
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

/// `boss.weapon_records` (local IR) -> records; the index in the result is the `Op::Spawn` record index.
pub fn lower_records(boss: &J) -> Vec<SpawnRecord> {
    let mut v = vec![];
    for r in boss.get("weapon_records").arr() {
        let a = |k: &str, i: usize| r.get(k).arr().get(i).map(jnum).unwrap_or(Num::K(0.0));
        let f = |k: &str, i: usize| r.get(k).arr().get(i).and_then(|x| x.num()).unwrap_or(0.0) as f32;
        let hb = r.get("hitbox");
        let hitbox = if hb.is_null() {
            SpawnBox::UNREAD
        } else {
            SpawnBox { damage: jnum(hb.get("damage")), angle: hb.get("angle").num().unwrap_or(45.0) as i32, kbg: jnum(hb.get("knockback_growth")), bkb: jnum(hb.get("base_knockback")), size: hb.get("size").num().unwrap_or(4.0) as f32 }
        };
        v.push(SpawnRecord {
            raw_id: r.get("raw_id").num().unwrap_or(0.0) as u32,
            at_owner: r.get("mode").str() == "owner",
            life: r.get("life").num().unwrap_or(60.0) as i32,
            scale: r.get("scale").num().unwrap_or(1.0) as f32,
            bone: r.get("bone").num().unwrap_or(-1.0) as i16,
            speed: [a("speed", 0), a("speed", 1)],
            angle: [a("angle", 0), a("angle", 1)],
            accel: [f("accel", 0), f("accel", 1)],
            offset: [f("offset", 0), f("offset", 1)],
            hitbox,
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
        assert_eq!(parse_spawn_text("650E vabf6 0 26"), Some((44022, 26)));
        assert_eq!(parse_spawn_text("650E vabee 0"), Some((44014, -1)));
        assert_eq!(parse_spawn_text("650E param44006"), Some((44006, -1)));
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
        let targets: [Target; 4] = [Some((target.0, target.1, 5.0)), None, None, None];
        let mut out = [(0, 0, 0.0, 0, 0.0, 0.0); MAX_SPAWN];
        let (mut spawned, mut hits, mut ends) = (0, 0, 0);
        let mut max_live = 0;
        for _ in 0..(len + 400) {
            gb::tick(&d, w);
            let nh = tick(&d, &targets, &mut out);
            hits += nh;
            let s = unsafe { S };
            for &(k, _) in s.ev.iter().take(s.ev_n as usize) {
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
    }

    fn tiny_define(spawns: Vec<SpawnRecord>, m: gb::Motion) -> Define {
        Define {
            name: "spawn-test".into(),
            parts: vec![gb::Part { name: "core".into(), role: gb::Role::Core, hp: Num::K(100.0), on_damaged: None, thresholds: vec![], on_zero: None, intangible_in: 0, int_level: false }],
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
        run_ir("galleom.brawl.ir.json", "Grenade", 4, (40.0, 8.0));
        run_ir("galleom.brawl.ir.json", "Shoot", 4, (60.0, -10.0));
    }

    #[test]
    fn duon_shots() {
        let Some((j, _)) = ir_motions("duon.brawl.ir.json") else { return };
        // every timeline with a spawn op fires and its shots are all removed
        for t in j.get("boss").get("timelines").arr() {
            let n = t.get("events").arr().iter().filter(|e| parse_spawn_text(e.get("op").get("text").str()).is_some()).count();
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
        let mut out = [(0, 0, 0.0, 0, 0.0, 0.0); MAX_SPAWN];
        let (mut sp, mut en) = (0, 0);
        for _ in 0..(len + 400) {
            gb::tick(&d, World::default());
            tick(&d, &[None; 4], &mut out);
            let s = unsafe { S };
            for &(e, _) in s.ev.iter().take(s.ev_n as usize) {
                sp += (e == EV_SPAWN) as usize;
                en += (e == EV_SPAWN_END) as usize;
            }
        }
        eprintln!("duon {motion}: {n} spawn ops, {k} lowered, spawned {sp}, removed {en}");
        assert_eq!(sp, k);
        assert_eq!(en, k);
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
        let mut out = [(0, 0, 0.0, 0, 0.0, 0.0); MAX_SPAWN];
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
    }
}
