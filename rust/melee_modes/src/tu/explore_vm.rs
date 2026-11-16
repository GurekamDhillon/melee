//! The Subspace map rooms' enemies run by the enemy-command VM (geno_vm: Brawl's own enemy scripts, compiled from the
//! owner's files), for every Brawl enemy kind that has a VM program and no hand-made Subspace actor (`subspace.rs`
//! keeps Goomba, Jyk, Giant Goomba, Bullet Bill, Hammer Bro, Primid and Koopa). Spawned from the room's generator
//! points through `explore::ENEMY_HOOK`.
//!
//! * Sim: a pool of [`N`] VM actors and their host records, plain statics (the rollback snapshot covers them), no
//!   allocation in the frame. Inputs are the players' places, the live collision under / beside the actor, the
//!   placement point and path, the model's root motion and this frame's damage. The program and the model are read
//!   once at the stage load (before the first frame, so every snapshot has them).
//! * Hits: the players' fighter and item hitboxes against a hurt capsule made from the model's rest extent (our
//!   choice: Brawl's hurt shapes are not read yet); the actor's forwarded attack boxes (0619 / 0604) against the
//!   players' bodies through Melee's own damage path (`subspace::strike`).
//! * Gone: when the VM removes the actor (6500), when its HP has been 0 for [`DEAD_HOLD`] frames, or out of the
//!   blast zone.
//!
//! `MELEE_SSE_VM=0` turns the lane off (those kinds are then listed, not spawned, or stand in with
//! `MELEE_SSE_STANDINS=1`).
#![allow(static_mut_refs, clippy::missing_safety_doc)]

use brawl_boss::vm::{self as vr, Program};
use melee_core::ppc::{fctiwz_f32, Ppc};
use melee_types::*;

pub const N: usize = 8;
const BOXES: usize = 4;
/// frames an actor stays after its HP reached 0 without removing itself (its dying motion)
const DEAD_HOLD: i32 = 90;
/// how far below its feet a floor still holds a walking actor (a slope or a step down)
const STEP: f32 = 4.0;

#[derive(Clone, Copy)]
pub struct Slot {
    /// `vm::FILES` row, -1 free
    pub file: i32,
    pub sec: i32,
    /// the map spawn index (its placement path), and the GD stage
    pub spawn: i32,
    pub stage: i32,
    pub home: [f32; 2],
    pub dead: i32,
    pub flash: i32,
    pub dmg: f32,
    /// the host's fall speed (see the fall in `tick`)
    pub vy: f32,
    /// per player slot, entity and hitbox: this hitbox already landed on us
    pub latch: [[[bool; 4]; 2]; 4],
    pub boxes: [crate::tu::geno_vm_host::VmBox; BOXES],
    /// per box: the players it has hit (bit per slot)
    pub hit: [u8; BOXES],
    pub inst: u32,
}

const NOBOX: crate::tu::geno_vm_host::VmBox = crate::tu::geno_vm_host::VmBox { on: false, id: 0, bone: 0, damage: 0.0, angle: 0, kbg: 0.0, wkb: 0.0, bkb: 0.0, size: 0.0, offset: [0.0; 3], inst: 0 };
const FREE: Slot = Slot { file: -1, sec: -1, spawn: -1, stage: 0, home: [0.0; 2], dead: 0, flash: 0, dmg: 0.0, vy: 0.0, latch: [[[false; 4]; 2]; 4], boxes: [NOBOX; BOXES], hit: [0; BOXES], inst: 0 };

pub static mut S: [Slot; N] = [FREE; N];
pub static mut A: [vr::Actor; N] = [vr::OFF; N];
/// the item hitboxes' latches: (item entity, per hitbox a bit per pool slot)
static mut LATCH_IT: [(usize, [u8; 4]); 16] = [(0, [0; 4]); 16];

/// The `vm::FILES` name of a Brawl enemy kind (sora_enemy's createInstance order; explore::enemy_kind's names), for the
/// kinds a VM program exists for.
pub fn vm_name(kind: u16) -> Option<&'static str> {
    Some(match kind {
        0 => "goomba",
        1 => "poppant",
        2 => "feyesh",
        3 => "jyk",
        4 => "auroros",
        5 => "cymul",
        6 => "roturret",
        7 => "borboras",
        8 => "giant-goomba",
        9 => "buckot",
        10 => "bucculus",
        11 => "greap",
        12 => "shaydas",
        13 => "bullet-bill",
        14 => "roader",
        15 => "spaak",
        16 => "mite",
        17 => "ticken",
        18 => "towtow",
        19 => "hammer-bro",
        20 => "bytan",
        21 => "floow",
        22 => "puppit",
        23 => "primid",
        24 => "shellpod",
        25 => "koopa",
        26 => "armight",
        27 => "bombed",
        29 => "nagagog",
        30 => "trowlon",
        32 => "boom-primid",
        35 => "sword-primid",
        36 => "gamyga",
        37 => "rob-blaster",
        38 => "rob-distance",
        39 => "rob-launcher",
        40 => "rob-sentry",
        41 => "autolance",
        42 => "armank",
        43..=45 => "glunder",
        _ => return None,
    })
}

/// A Brawl kind this lane runs (a VM program loaded for it at the stage load).
pub fn has_vm_kind(kind: u16) -> bool {
    enabled() && !crate::tu::explore::has_actor(kind) && vm_name(kind).and_then(row).is_some_and(|r| prog(r as i32).is_some())
}

pub fn enabled() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var("MELEE_SSE_VM").map_or(true, |v| v != "0"))
}

const PMAX: usize = 64;
/// The room programs: the compiled program with the boss arena box lifted (a room's collision bounds it instead),
/// made at the stage load, one per `vm::FILES` row.
static PROGS: [std::sync::OnceLock<Option<&'static Program>>; PMAX] = [const { std::sync::OnceLock::new() }; PMAX];

fn row(name: &str) -> Option<usize> {
    vr::FILES.iter().position(|f| f.0 == name).filter(|&k| k < PMAX)
}

/// The stage load (before the first frame): compile and load every VM kind among the map's enemy points. Kinds with
/// their own Subspace actor are skipped (unless the stand-ins are on, they stay those actors either way).
pub fn prefetch(kinds: &[u16]) {
    if !enabled() {
        return;
    }
    let names: std::vec::Vec<&'static str> = kinds.iter().filter(|&&k| !crate::tu::explore::has_actor(k)).filter_map(|&k| vm_name(k)).collect();
    // the kinds side by side (program then model, each kind on a thread of its own): read one after another they
    // were most of the match's start tick when the match began right at the pick
    let t = std::time::Instant::now();
    std::thread::scope(|sc| {
        for &name in &names {
            sc.spawn(move || {
                crate::tu::geno_vm::prefetch(name);
                crate::tu::vm_boss_model::enemy_prefetch(name);
            });
        }
    });
    eprintln!("explore: {} room enemy kinds read in {} ms", names.len(), t.elapsed().as_millis());
    for &name in &names {
        let Some(r) = row(name) else { continue };
        PROGS[r].get_or_init(|| {
            crate::tu::geno_vm::program(name)?;
            crate::tu::vm_boss_model::enemy_prefetch(name);
            crate::tu::vm_boss_model::enemy_get(name)?;
            // the motion lengths the model set are in the program now
            let mut p = crate::tu::geno_vm::program(name)?.clone();
            p.bounds = [-1.0e6, 1.0e6, -1.0e6, 1.0e6];
            let p: &'static Program = Box::leak(Box::new(p));
            Some(p)
        });
    }
}

/// Queue the loaded VM enemy models' textures among `kinds` for upload before the match (the map's own are queued at
/// the pick): their first draw otherwise uploads them mid-match.
pub fn queue_textures(kinds: &[u16]) {
    // the warm-up list only: queueing these models' textures for the preload crashed the synctest (exit 127, Lake
    // depth 7); their textures go up with the warm-up draws instead, one model every other frame
    let mut seen: std::vec::Vec<i32> = vec![];
    for &k in kinds {
        let Some(file) = vm_name(k).filter(|_| has_vm_kind(k)).and_then(row).map(|r| r as i32) else { continue };
        if seen.contains(&file) || model(file).is_none() {
            continue;
        }
        seen.push(file);
        if let Ok(mut w) = WARM.lock() {
            w.push(file);
        }
    }
}

fn prog(file: i32) -> Option<&'static Program> {
    *PROGS.get(usize::try_from(file).ok()?)?.get()?
}

fn model(file: i32) -> Option<&'static crate::tu::vm_boss_model::Model> {
    crate::tu::vm_boss_model::enemy_get(vr::FILES.get(usize::try_from(file).ok()?)?.0)
}

/// The match begins: an empty pool.
pub unsafe fn reset() {
    S = [FREE; N];
    A = [vr::OFF; N];
    LATCH_IT = [(0, [0; 4]); 16];
}

/// A door to another section: the room's VM enemies go.
pub unsafe fn clear_room() {
    reset();
}

/// Checks only (`MELEE_EXPLORE_BEAT`): every live VM enemy is set below the blast zone, so the next tick takes it
/// as beaten (a fall out is a defeat).
pub unsafe fn test_drop_all() {
    for i in 0..N {
        if S[i].file >= 0 {
            A[i].pos[1] = -1.0e5;
        }
    }
}

pub fn alive() -> usize {
    unsafe { S.iter().filter(|s| s.file >= 0).count() }
}

/// `explore::ENEMY_HOOK`: spawn point `spawn` of GD stage `stage` (Brawl kind `kind`, at x, y, facing, section `sec`)
/// as a VM actor. False: not a VM kind here (the caller lists it or stands one in).
pub unsafe fn spawn(kind: u16, x: f32, y: f32, face: i32, sec: usize, stage: i32, spawn: usize) -> bool {
    if !enabled() || crate::tu::explore::has_actor(kind) {
        return false;
    }
    let Some(name) = vm_name(kind) else { return false };
    let Some(file) = row(name) else { return false };
    let Some(p) = prog(file as i32) else { return false };
    let Some(i) = S.iter().position(|s| s.file < 0) else { return false };
    // the generator record's start status (a Koopa on its flight path, a Goomba falling in), else the IR's / 0
    let st = crate::tu::explore::spawn_status(stage, spawn)
        .filter(|&s| s >= 0 && (s as usize) < p.status_enter.len())
        .or_else(|| crate::tu::geno_boss_ir::ir_start_status(&format!("{}.brawl.ir.json", vr::FILES[file].1)))
        .unwrap_or(0);
    // the actor's own generator: from its place and slot (no wall clock)
    let seed = x.to_bits() ^ y.to_bits().rotate_left(13) ^ (i as u32).wrapping_mul(0x9E37_79B9) | 1;
    vr::begin(p, &mut A[i], 1, 5, seed, st, true);
    A[i].pos = [x, y, 0.0];
    A[i].lr = if face < 0 { -1.0 } else { 1.0 };
    // in the neutral (gameplay) hash, so a synctest / netplay check sees the actors themselves
    if melee_fighter::tu::fighter::BOSS_HASH.is_none() {
        melee_fighter::tu::fighter::BOSS_HASH = Some(hash_word);
    }
    // drawn in the Subspace pool's fighter draw pass (the hook a boss stage keeps for itself)
    if melee_fighter::tu::ftdrawcommon::BOSS_DRAW.is_none() {
        melee_fighter::tu::ftdrawcommon::BOSS_DRAW = Some(crate::tu::subspace::sub_draw);
    }
    S[i] = Slot { file: file as i32, sec: sec as i32, spawn: spawn as i32, stage, home: [x, y], ..FREE };
    eprintln!("explore: enemy {name} (Brawl kind {kind}) by the VM at ({x:.0},{y:.0}) section {sec} -> vm slot {i} status {st}");
    true
}

/// The rest extent of the model as a hurt capsule (bottom, top, radius), model units at scale 1.
fn hurt(i: usize) -> ((f32, f32), (f32, f32), f32) {
    let a = unsafe { &A[i] };
    let (lo, hi, w) = match model(unsafe { S[i].file }) {
        Some(m) => (m.rig.min[1], m.rig.max[1], 0.5 * (m.rig.max[2] - m.rig.min[2]).max(m.rig.max[0] - m.rig.min[0])),
        None => (0.0, 8.0, 4.0),
    };
    let r = (0.5 * w).clamp(2.0, 30.0);
    let (b, t) = (a.pos[1] + lo + r, a.pos[1] + hi - r);
    ((a.pos[0], b.min(t)), (a.pos[0], t.max(b)), r)
}

unsafe fn input(i: usize) -> vr::Input {
    use melee_world::tu::mplib as mp;
    let a = &A[i];
    let s = &S[i];
    let mut inp = vr::Input { damage: s.dmg, home: Some(s.home), enemy_room: false, ..Default::default() };
    for k in 0..crate::tu::subspace::humans() {
        let f = crate::tu::subspace::target(k);
        if !f.is_null() {
            inp.targets[k as usize] = Some([(*f).cur_pos.x, (*f).cur_pos.y]);
            inp.target_lr[k as usize] = (*f).facing_dir;
            inp.target_scale[k as usize] = (*f).x34_scale.y;
        }
    }
    let (x, y) = (a.pos[0], a.pos[1]);
    let (mut p, mut n): (Vec, Vec) = (core::mem::zeroed(), core::mem::zeroed());
    let (mut line, mut flags) = (-1i32, 0u32);
    if mp::mpCheckFloor(x, y + 10.0, x, y - 400.0, 0.0, &mut p, &mut line, &mut flags, &mut n, -1, -1, -1, None, core::ptr::null_mut()) != 0 {
        inp.floor_on = true;
        inp.floor_y = p.y;
        inp.ground = (y - p.y).abs() <= 1.0;
    }
    let yh = y + 10.0;
    for (side, dx) in [(0usize, -400.0f32), (1, 400.0)] {
        for f in [mp::mpCheckLeftWall, mp::mpCheckRightWall] {
            if f(x, yh, x + dx, yh, &mut p, &mut line, &mut flags, &mut n, -1, -1) != 0 && (p.x - x) * dx >= 0.0 {
                if inp.wall_on & 1u32.pshl(side as i64) == 0 || (p.x - x).abs() < (inp.wall[side] - x).abs() {
                    inp.wall[side] = p.x;
                    inp.wall_on |= 1u32.pshl(side as i64);
                }
            }
        }
    }
    // the ground module's surface queries as the boss host reads them: on a floor; the floor goes on ahead (a floor
    // under the point 6 units ahead)
    if inp.ground {
        let ahead = x + if a.lr >= 0.0 { 6.0 } else { -6.0 };
        let more = mp::mpCheckFloor(ahead, y + 10.0, ahead, y - 20.0, 0.0, &mut p, &mut line, &mut flags, &mut n, -1, -1, -1, None, core::ptr::null_mut()) != 0;
        inp.ground_q = 1 | if more { 2 } else { 0 };
        let side = if a.lr >= 0.0 { 1 } else { 0 };
        inp.cliff[side] = !more;
    }
    inp.offscreen = crate::tu::subspace::out_of_bounds(x, y);
    if let Some((path, _)) = crate::tu::explore::enemy_vm_place(s.stage, s.spawn as usize) {
        inp.path = path;
    }
    if let (Some(m), Some(p)) = (model(s.file), prog(s.file)) {
        inp.root_on = true;
        inp.root_move = crate::tu::vm_boss_model::root_move_m(m, p, a);
        inp.root_scale = 1.0;
    }
    inp
}

/// One room frame of the pool (explore's room tick, once per logic frame).
pub unsafe fn tick() {
    if S.iter().all(|s| s.file < 0) {
        return;
    }
    player_hits();
    for i in 0..N {
        if S[i].file < 0 {
            continue;
        }
        let Some(p) = prog(S[i].file) else {
            S[i] = FREE;
            continue;
        };
        let mut inp = input(i);
        // the situation module as the ground module leaves it (Brawl's ground / air switch is the engine's, not the
        // scripts'): on a floor within a step below the feet the actor stands on it (slopes, steps down), else it is
        // in the air and its scripts' fall rules take it [L: the step height is ours]
        if inp.floor_on && A[i].pos[1] - inp.floor_y <= STEP && A[i].pos[1] >= inp.floor_y - 0.5 && A[i].situation == 0 {
            A[i].pos[1] = inp.floor_y;
            inp.ground = true;
        }
        // only the engine's own switches: walked off a floor (ground -> air), came down on one (air -> ground); a
        // script's own 0900 (a jump) stands otherwise
        if A[i].situation == 0 && !inp.ground {
            A[i].situation = 1;
        } else if A[i].situation != 0 && inp.ground && S[i].vy < 0.0 {
            A[i].situation = 0;
        }
        S[i].dmg = 0.0;
        if S[i].flash > 0 {
            S[i].flash -= 1;
        }
        let y0 = A[i].pos[1];
        vr::step(p, &mut A[i], &inp);
        // the fall: the VM's built-in gravity is 0 (engine.md 6), so in a falling or damage motion with no movement of
        // its own on y the host lets it fall to the floor [L: our gravity, a Melee-like 0.1 a frame up to 2.5]
        {
            let a = &mut A[i];
            let mn = p.motion_names.get(a.motion.max(0) as usize).map(String::as_str).unwrap_or("");
            let falling = a.situation != 0 && a.pos[1] == y0 && (mn.contains("Fall") || mn.contains("Damage"));
            if falling {
                S[i].vy = (S[i].vy - 0.1).max(-2.5);
                let ny = a.pos[1] + S[i].vy;
                a.pos[1] = if inp.floor_on && ny <= inp.floor_y { inp.floor_y } else { ny };
            } else if a.situation == 0 {
                S[i].vy = 0.0;
            }
        }
        if A[i].tick % 120 == 1 && std::env::var_os("MELEE_EXPLORE_LOG").is_some() {
            let a = &A[i];
            let mn = p.motion_names.get(a.motion.max(0) as usize).map(String::as_str).unwrap_or("?");
            eprintln!("explore: vm slot {i} {} t{} status {} motion {mn} at ({:.1},{:.1}) hp {:.1} ground {} floor {:?}", vr::FILES[S[i].file as usize].0, a.tick, a.status, a.pos[0], a.pos[1], a.hp(), inp.ground, inp.floor_on.then_some(inp.floor_y));
        }
        let mut removed = false;
        let a = A[i];
        for e in a.events() {
            match e.kind {
                vr::EV_REMOVE => removed = true,
                vr::EV_STATUS | vr::EV_MOTION => {
                    for b in S[i].boxes.iter_mut() {
                        b.on = false;
                    }
                }
                vr::EV_CMD => {
                    let Some(ins) = p.code.get(e.ins as usize) else { continue };
                    if std::env::var_os("MELEE_EXPLORE_CMDLOG").is_some() {
                        eprintln!("explore: vm slot {i} cmd {:04X} n {} status {}", ins.id, ins.n, a.status);
                    }
                    match ins.id {
                        0x0619 => {
                            S[i].inst = S[i].inst.wrapping_add(1);
                            if let Some(nb) = crate::tu::geno_vm_host::box_of(p, &a, ins, S[i].inst) {
                                let bx = &mut S[i].boxes;
                                let k = bx.iter().position(|b| b.on && b.id == nb.id).or_else(|| bx.iter().position(|b| !b.on));
                                if let Some(k) = k {
                                    if std::env::var_os("MELEE_EXPLORE_LOG").is_some() {
                                        eprintln!("explore: vm slot {i} box {} on: bone {} {:.1} dmg size {:.1}", nb.id, nb.bone, nb.damage, nb.size);
                                    }
                                    bx[k] = nb;
                                    S[i].hit[k] = 0;
                                }
                            }
                        }
                        0x0604 => {
                            for b in S[i].boxes.iter_mut() {
                                b.on = false;
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        if a.hp() <= 0.0 {
            S[i].dead += 1;
        }
        if removed || S[i].dead > DEAD_HOLD || crate::tu::subspace::out_of_bounds(a.pos[0], a.pos[1]) {
            if let Some(m) = crate::tu::explore::map_of(S[i].stage) {
                // a removal at 0 HP or a fall out is a defeat (its trigger fires); a script's own removal alive is not
                crate::tu::explore::enemy_gone(m, S[i].spawn as usize, S[i].dead > 0 || !removed);
            }
            eprintln!("explore: vm enemy {} (slot {i}) gone at ({:.0},{:.0}){}", vr::FILES[S[i].file as usize].0, a.pos[0], a.pos[1], if removed { "" } else if S[i].dead > DEAD_HOLD { ", beaten" } else { ", out of bounds" });
            S[i] = FREE;
            A[i] = vr::OFF;
            continue;
        }
        if S[i].dead == 0 {
            attack_players(i, p);
        }
    }
}

/// The actor's live boxes against the players' bodies (each box hits a player once).
unsafe fn attack_players(i: usize, p: &Program) {
    let a = &A[i];
    let m = model(S[i].file);
    for k in 0..BOXES {
        let b = S[i].boxes[k];
        if !b.on || b.damage <= 0.0 {
            continue;
        }
        let c = m.and_then(|m| crate::tu::vm_boss_model::bone_world_m(m, p, a, b.bone, b.offset)).unwrap_or([a.pos[0], a.pos[1]]);
        for s in 0..crate::tu::subspace::humans() {
            let bit = 1u8.pshl(s as i64);
            if S[i].hit[k] & bit != 0 {
                continue;
            }
            let f = crate::tu::subspace::target(s);
            if f.is_null() {
                continue;
            }
            let (ba, bb, br) = crate::tu::subspace::body_of(f);
            if crate::tu::subspace::seg_dist((c[0], c[1]), (c[0], c[1]), ba, bb) <= br + b.size {
                S[i].hit[k] |= bit;
                if std::env::var_os("MELEE_EXPLORE_LOG").is_some() {
                    eprintln!("explore: vm enemy {} (slot {i}) hits player {s}: {:.1} damage", vr::FILES[S[i].file as usize].0, b.damage);
                }
                crate::tu::subspace::strike(f, a.pos[0], fctiwz_f32(b.damage), b.angle, fctiwz_f32(b.kbg), fctiwz_f32(b.wkb), fctiwz_f32(b.bkb));
            }
        }
    }
}

unsafe fn hit_pool(a: (f32, f32), b: (f32, f32), r: f32, dmg: f32, mut latched: impl FnMut(usize) -> bool) -> bool {
    let mut any = false;
    for i in 0..N {
        if S[i].file < 0 || S[i].dead > 0 || A[i].hurt == 2 || latched(i) {
            continue;
        }
        let (ha, hb, hr) = hurt(i);
        if crate::tu::subspace::seg_dist(a, b, ha, hb) <= r + hr {
            S[i].dmg += dmg;
            if std::env::var_os("MELEE_EXPLORE_LOG").is_some() {
                eprintln!("explore: vm enemy {} (slot {i}) hit for {dmg:.1} (hp {:.1})", vr::FILES[S[i].file as usize].0, A[i].hp());
            }
            S[i].flash = 4;
            any = true;
        }
    }
    any
}

/// The players' fighter hitboxes and their items' hitboxes against the pool (`subspace::player_hits`' rule).
unsafe fn player_hits() {
    use melee_fighter::tu::player as pl;
    for s in 0..crate::tu::subspace::humans() {
        for en in 0..2usize {
            let f = crate::tu::subspace::entity(s, en);
            if f.is_null() {
                continue;
            }
            let ratio = pl::Player_GetAttackRatio(s);
            for h_i in 0..4 {
                let h: *mut HitCapsule = core::ptr::addr_of_mut!((*f).x914[h_i]);
                if (*h).state == HitCapsule_Disabled {
                    for sl in S.iter_mut() {
                        sl.latch[s as usize][en][h_i] = false;
                    }
                    continue;
                }
                let r = if (*h).x43_b1() != 0 { (*h).scale } else { (*h).scale * (*f).x34_scale.y };
                let (a, b) = (((*h).x58.x, (*h).x58.y), ((*h).x4C.x, (*h).x4C.y));
                let dmg = (*h).damage * ratio;
                if dmg <= 0.0 {
                    continue;
                }
                let hit = hit_pool(a, b, r, dmg, |i| {
                    let l = &mut S[i].latch[s as usize][en][h_i];
                    let was = *l;
                    *l = true;
                    was
                });
                if hit {
                    let fp = &mut *f;
                    let d = fctiwz_f32(dmg + 0.5).max(1);
                    if (fp.dmg.x1914 as i32) < d {
                        fp.dmg.x1914 = d as _;
                    }
                    let snd = 0x7E + fp.player_idx as i32 * 2 + fp.is_sub_fighter() as i32;
                    melee_runtime::tu::lbcollision::lbColl_80005BB0(h, snd);
                }
            }
        }
    }
    let mut next: [(usize, [u8; 4]); 16] = [(0, [0; 4]); 16];
    let mut n = 0;
    let mut g = *melee_runtime::tu::gobj_native::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
    while !g.is_null() {
        let ip = (*g).user_data as *mut Item;
        g = (*g).next;
        if ip.is_null() {
            continue;
        }
        let it = &mut *ip;
        let owner = it.owner;
        if !owner.is_null() && (*owner).classifier == HSD_GOBJ_CLASS_FIGHTER as _ {
            let of = (*owner).user_data as *mut Fighter;
            if !of.is_null() && (*of).player_idx as i32 >= crate::tu::subspace::humans() {
                continue;
            }
        }
        let key = it.entity as usize;
        let mut lat = LATCH_IT.iter().find(|(k, _)| *k == key).map(|(_, l)| *l).unwrap_or([0; 4]);
        let mut live = false;
        for h_i in 0..4 {
            let h: *mut HitCapsule = core::ptr::addr_of_mut!(it.x5D4_hitboxes[h_i].hit);
            if (*h).state == HitCapsule_Disabled {
                lat[h_i] = 0;
                continue;
            }
            live = true;
            let (a, b) = (((*h).x58.x, (*h).x58.y), ((*h).x4C.x, (*h).x4C.y));
            let l = &mut lat[h_i];
            if (*h).damage > 0.0
                && hit_pool(a, b, (*h).scale, (*h).damage, |i| {
                    let bit = 1u8.pshl(i as i64);
                    let was = *l & bit != 0;
                    *l |= bit;
                    was
                })
            {
                melee_runtime::tu::lbcollision::lbColl_80005BB0(h, 0x7E);
            }
        }
        if live && n < next.len() {
            next[n] = (key, lat);
            n += 1;
        }
    }
    LATCH_IT = next;
}

/// Presentation: every live actor's model at its motion frame (called from the Subspace pool's draw pass).
/// Render-only (outside the snapshot): the picked stage's VM enemy model files not yet drawn once. The match's first
/// drawn frame draws each far off screen (plain and flashing), so their pipelines are built at the match start, not
/// on the frame an enemy first comes in.
static WARM: std::sync::Mutex<std::vec::Vec<i32>> = std::sync::Mutex::new(std::vec::Vec::new());

/// The map's draw (the match's first drawn frame on): build the warm list's pipelines.
pub unsafe fn warm_draw() {
    // one model a frame, so no frame builds them all
    let warm: Option<i32> = WARM.lock().ok().and_then(|mut w| w.pop());
    for f in warm {
        let (Some(m), Some(p)) = (model(f), prog(f)) else { continue };
        // a bare actor (no `begin`: running the program's start would touch the game) in its first motion
        let mut a = vr::OFF;
        a.pos = [1.0e5, 1.0e5, 0.0];
        crate::tu::vm_boss_model::draw_m(m, p, &a, false, (-1, 0), 1.0);
    }
}

pub unsafe fn draw() {
    for i in 0..N {
        let s = S[i];
        if s.file < 0 {
            continue;
        }
        let (Some(m), Some(p)) = (model(s.file), prog(s.file)) else { continue };
        let fade = if s.dead > DEAD_HOLD - 10 { ((DEAD_HOLD - s.dead) as f32 / 10.0).max(0.05) } else { 1.0 };
        crate::tu::vm_boss_model::draw_m(m, p, &A[i], s.flash > 0, (-1, 0), fade);
    }
}

/// The pool's state for the rollback checks' hash (every live actor's VM hash and host record).
pub fn state_hash() -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    unsafe {
        for i in 0..N {
            let s = &S[i];
            for v in [s.file as u64, s.dead as u64, s.dmg.to_bits() as u64, s.vy.to_bits() as u64, if s.file >= 0 { A[i].vm_hash() as u64 } else { 0 }] {
                h = (h ^ v).wrapping_mul(0x100_0000_01b3);
            }
        }
    }
    h
}

/// `fighter::BOSS_HASH`: the pool's hash folded to a word while any actor is live.
pub fn hash_word() -> Option<u32> {
    (alive() > 0).then(|| {
        let h = state_hash();
        (h as u32) ^ (h >> 32) as u32
    })
}
