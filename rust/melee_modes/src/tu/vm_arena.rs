//! A VM boss's arena actor: the Falcon Flyer under Meta Ridley (SSE stage 370301, Boss Battles).
//!
//! In Brawl the Flyer is not a boss. It is an enemy of its own (class emFalconflyer, no AI in its native class: the
//! constructor 0x81A64654) that the stage places beside Meta Ridley and links to him; its scripts only react. Its
//! rule 3 copies his attack-id variable through 6519 (the enemy manager's link, 0x81A09860) and, when the id changed
//! and is not -1, enters the status of that number (Rush, Wingslap, Clings, Slap, Press); its rule 4 sends it home
//! when the link reports him gone (R10053). Its deck is the floor of the fight: the file's grColDataSet puts the
//! collision points on the model's bones, so the floor moves with every reaction clip.
//!
//! Here the Flyer runs on the same enemy-command VM as the boss, as a second actor (`geno_boss::State::vm2`, so the
//! snapshot and NHV1 carry it), stepped right after the boss each frame with the boss's variables as its link
//! (`Input::partner_vars`). Its motion comes from its own scripts and the model's root tracks (vm_boss_model, its
//! own model slot); its deck is the line set of geno_boss_coll, moved to the posed deck bones each frame; it is
//! drawn with the boss. Everything is read from the player's files at run time; nothing here names a path or a
//! value from them.

use crate::tu::geno_boss::{MAX_COLL_PTS, S};
use brawl_boss::vm::{self, Program};
use brawl_boss::vm as vr;

/// The arena actor a VS boss brings (None: the boss fights alone).
pub fn partner_of(boss: &str) -> Option<&'static str> {
    match boss {
        "metaridley" => Some("falconflyer"),
        _ => None,
    }
}

/// A name that is an arena actor, never a boss on its own.
pub fn is_arena_actor(name: &str) -> bool {
    name == "falconflyer"
}

/// This process's arena actor (fixed: the VS boss's).
pub fn partner() -> Option<&'static str> {
    crate::tu::petey::vs_boss().and_then(partner_of)
}

/// Its program (the files compiled on the engine-start helper; None without them).
pub fn program() -> Option<&'static Program> {
    static P: std::sync::OnceLock<Option<&'static Program>> = std::sync::OnceLock::new();
    *P.get_or_init(|| {
        let name = partner()?;
        let _ = crate::tu::vm_boss_model::get(name);
        // the latest copy: the model loader replaces it with the clip lengths
        let p = crate::tu::geno_vm::program(name)?;
        let mut q = p.clone();
        // no arena box: the ship's own clips carry it (ClingsAttackLoop's root track sinks it once per loop and
        // ClingsAttackEnd's lifts it back by the same three loops' worth), and the deepest dip goes below the stage
        // file's bottom blast line; clamping it there left it high after the lift
        q.bounds = [-2000.0, 2000.0, -2000.0, 2000.0];
        Some(&*Box::leak(Box::new(q)))
    })
}

/// The deck: the IR's platform joints (MDL0 bone indices), left to right on the deck's upper envelope as the model
/// stands in its Wait clip. Made once, ahead of the match (the engine-start helper).
fn deck() -> Option<&'static [i32]> {
    static D: std::sync::OnceLock<Option<Vec<i32>>> = std::sync::OnceLock::new();
    D.get_or_init(|| {
        let name = partner()?;
        let r = std::thread::spawn(move || -> Result<Vec<i32>, String> {
            let pc = crate::tu::geno_boss_coll::ir_joints(name)?;
            let m = crate::tu::vm_boss_model::get(name).ok_or("no model")?;
            let idx: Vec<u32> = pc.joints.iter().map(|j| j.strip_prefix("bone").and_then(|k| k.parse().ok()).ok_or_else(|| format!("joint {j} is not a bone index"))).collect::<Result<_, _>>()?;
            let slots: Vec<usize> = idx.iter().map(|&k| m.rig.bone_by_index(k).ok_or_else(|| format!("bone {k} not in the model"))).collect::<Result<_, _>>()?;
            let refa = m.rig.anim("Wait");
            // the model faces along its z: the side view is (z, y)
            let pts: Vec<[f32; 2]> = slots.iter().map(|&s| { let p = m.rig.bone_point(refa, 0.0, s, [0.0; 3]); [p[2], p[1]] }).collect();
            let order = crate::tu::geno_boss_coll::upper_envelope(&pts);
            if order.len() < 2 || order.len() > MAX_COLL_PTS {
                return Err(format!("{} deck points", order.len()));
            }
            Ok(order.into_iter().map(|i| idx[i] as i32).collect())
        })
        .join()
        .unwrap_or_else(|_| Err("deck reader panicked".into()));
        match r {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("vm_arena: WARNING: {name}: no deck ({e})", name = partner().unwrap_or("?"));
                None
            }
        }
    })
    .as_deref()
}

/// Engine start (helper thread): compile the Flyer, load its model, read its deck.
pub fn prefetch() {
    if let Some(n) = partner() {
        crate::tu::geno_vm::prefetch(n);
        crate::tu::vm_boss_model::prefetch(n);
        let _ = deck();
        let _ = backdrop();
    }
}

/// The fight's own background: the boss archive's stage model (Meta Ridley's 370301 holds one besides the stage
/// position model), the MDL0 whose name starts with the level's 3-digit prefix and `vAdv`, in its bind pose. Loaded
/// once on the engine-start helper. None: the archive has none (the stage's clear colour shows).
fn backdrop() -> Option<&'static (String, crate::tu::petey_model::Rig, Vec<crate::tu::petey_model::Vert>)> {
    static B: std::sync::OnceLock<Option<(String, crate::tu::petey_model::Rig, Vec<crate::tu::petey_model::Vert>)>> = std::sync::OnceLock::new();
    B.get_or_init(|| {
        let name = partner()?;
        let arc = brawl_boss::vm::FILES.iter().find(|f| f.0 == name)?.2;
        let r = std::thread::spawn(move || {
            let leaves = crate::tu::petey_model::pac_leaves(arc)?;
            // the background's MDL0: named "<3 digits>vAdv..." (the stage position model is "StagePosition")
            let mut names = vec![];
            for b in &leaves {
                if b.len() >= 0x10 && &b[..4] == b"bres" {
                    for (n, o) in crate::tu::petey_model::brres_entries(b).unwrap_or_default() {
                        if b.get(o..o + 4) == Some(b"MDL0") && n.len() > 7 && n.as_bytes()[..3].iter().all(u8::is_ascii_digit) && n[3..].starts_with("vAdv") && !names.contains(&n) {
                            names.push(n);
                        }
                    }
                }
            }
            let want: Vec<&str> = names.iter().map(String::as_str).collect();
            crate::tu::petey_model::static_models(&leaves, &want).into_iter().next()
        })
        .join()
        .ok()
        .flatten();
        match &r {
            Some((n, _, t)) => eprintln!("vm_arena: backdrop {n} ({} vertices)", t.len()),
            None => eprintln!("vm_arena: no backdrop model in the boss archive; the stage's clear colour shows"),
        }
        r
    })
    .as_ref()
}

/// Final Destination's own model (and its background) is not drawn while the arena runs (grdisplay::HIDE_GROUND, via
/// explore::hide_ground): the fight's stage is the ship over the boss archive's backdrop. Presentation only.
pub fn stage_hidden() -> bool {
    crate::tu::geno_boss_coll::present() && live()
}

/// Before the stage loads (the VS setup): ask geno_boss_coll for the deck's line set. False: the arena is missing
/// (logged), the boss fights without it.
pub fn want() -> bool {
    let Some(name) = partner() else { return false };
    if program().is_none() {
        eprintln!("vm_arena: WARNING: {name}: the arena actor's scripts did not load; the boss fights without it");
        return false;
    }
    match deck() {
        Some(d) => {
            // the deck is the fight's only ground: Final Destination's floor and edges are off for the match
            crate::tu::geno_boss_coll::want_alone(d.len(), true);
            eprintln!("vm_arena: {name}: arena actor ready, deck of {} points", d.len());
            true
        }
        None => false,
    }
}

/// The Flyer's start: its Wait status (s2), which places it at its home point.
pub fn begin(difficulty: i32, seed: u32) {
    let Some(p) = program() else { return };
    unsafe {
        let s = &mut *core::ptr::addr_of_mut!(S);
        vm::begin(p, &mut s.vm2, 1, difficulty.clamp(0, 14), seed ^ 0x5F1E_7E12, 2, true);
        BOXES = [OFF_BOX; MAX_BOXES];
        INST = 0;
    }
    let a = actor();
    eprintln!("vm_arena: {} begins (status {}, at {:.1} {:.1})", partner().unwrap_or("?"), a.status, a.pos[0], a.pos[1]);
    // MELEE_PETEY_LOG: each clip's root track (start, end, lowest, highest up), the motion the scripts' kind-1 units take
    if std::env::var_os("MELEE_PETEY_LOG").is_some() {
        if let Some(m) = crate::tu::vm_boss_model::arena_model() {
            for (k, t) in m.root.iter().enumerate() {
                if let (Some(f), Some(l)) = (t.first(), t.last()) {
                    let lo = t.iter().map(|v| v[1]).fold(f32::MAX, fmin);
                    let hi = t.iter().map(|v| v[1]).fold(f32::MIN, fmax);
                    eprintln!(
                        "vm_arena: motion {k} {} ({} frames, loop {}): root fwd {:.1} -> {:.1}, up {:.1} -> {:.1} (range {:.1}..{:.1})",
                        p.motion_names.get(k).map_or("?", |s| s.as_str()), t.len(), p.motion_loop.get(k).copied().unwrap_or(false), f[0], l[0], f[1], l[1], lo, hi
                    );
                }
            }
        }
    }
}

pub fn actor() -> &'static vr::Actor {
    unsafe { &*core::ptr::addr_of!(S.vm2) }
}

pub fn live() -> bool {
    actor().live != 0
}

/// One frame, after the boss's: the link in, the Flyer stepped, its deck moved.
pub unsafe fn arena_frame(frame: u32, log: bool) {
    let Some(p) = program() else { return };
    if !live() {
        return;
    }
    let s = &mut *core::ptr::addr_of_mut!(S);
    let boss = &s.vm;
    let mut i = vr::Input { linked: true, ..Default::default() };
    // R10053 (partner gone): the boss's enemy is gone or its dead flag is set
    i.partner_alive = boss.live != 0 && !boss.dead();
    i.partner_status = (boss.live != 0).then_some(boss.status);
    i.partner_vars = Some(vr::PartnerVars::of(boss));
    // no floor or walls: the ship flies (its moves stay in the arena box)
    if let Some(m) = crate::tu::vm_boss_model::arena_model() {
        if let Some(mv) = crate::tu::vm_boss_model::root_move_m(m, p, &s.vm2) {
            i.root_on = true;
            i.root_move = mv;
            i.root_scale = 1.0;
        }
    }
    let last = s.vm2.status;
    vm::step(p, &mut s.vm2, &i);
    let a = &s.vm2;
    // the ship's attack boxes (its forwarded 0619 / 0604): a status or motion change ends them, as the boss's
    for e in a.events() {
        match e.kind {
            vr::EV_STATUS | vr::EV_MOTION => BOXES = [OFF_BOX; MAX_BOXES],
            vr::EV_CMD => match p.code.get(e.ins as usize).map(|c| c.id) {
                Some(0x0619) => set_box(p, a, &p.code[e.ins as usize], frame, log),
                Some(0x0604) => BOXES = [OFF_BOX; MAX_BOXES],
                _ => {}
            },
            _ => {}
        }
    }
    if log {
        for e in a.events() {
            match e.kind {
                vr::EV_STATUS => eprintln!(
                    "vm_arena: f{frame} flyer status {last} -> {} at {:.1} {:.1} (boss attack id {})",
                    e.v, a.pos[0], a.pos[1], boss.wi[0x14]
                ),
                vr::EV_CMD => {
                    if p.code.get(e.ins as usize).is_some_and(|c| c.id == 0x6519) {
                        // the link copy ran (the value is in the actor already); logged on a change only
                        if a.wi[0x14] != a.wi[0x06] {
                            eprintln!("vm_arena: f{frame} flyer link: attack id {}", a.wi[0x14]);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    move_deck(p);
    if log && frame % 120 == 0 {
        let n = s.coll_n as usize;
        if n >= 2 {
            eprintln!(
                "vm_arena: f{frame} flyer status {} at {:.1} {:.1}, deck ({:.1},{:.1})..({:.1},{:.1})",
                a.status, a.pos[0], a.pos[1], s.coll_pts[0][0], s.coll_pts[0][1], s.coll_pts[n - 1][0], s.coll_pts[n - 1][1]
            );
        }
    }
}

use crate::tu::geno_vm_host::VmBox;
const MAX_BOXES: usize = 4;
const OFF_BOX: VmBox = VmBox { on: false, id: 0, bone: 0, damage: 0.0, angle: 0, kbg: 0.0, wkb: 0.0, bkb: 0.0, size: 0.0, offset: [0.0; 3], inst: 0 };
/// The ship's live attack boxes (a static of the executable: the snapshot carries it) and their numbering.
static mut BOXES: [VmBox; MAX_BOXES] = [OFF_BOX; MAX_BOXES];
static mut INST: u32 = 0;

/// A ship 0619: SlapAttack's box (id 0, frames 84-85) and the 0-damage box its Rush / Wait / Wingslap timelines call
/// (id 3). The same box set again while it is still on keeps its hit group (the Wait clip restarts every 12 frames and
/// sets id 3 each time; a new group each time would re-hit every rider five times a second) [L: Brawl's re-set rule
/// is not read]. The 0603 those timelines also run is not read [L] and does nothing here.
unsafe fn set_box(p: &'static Program, a: &vr::Actor, ins: &vr::Ins, frame: u32, log: bool) {
    let Some(mut nb) = crate::tu::geno_vm_host::parse_box(p, a, ins) else { return };
    let b = &mut *core::ptr::addr_of_mut!(BOXES);
    let same = b.iter().position(|x| x.on && x.id == nb.id);
    if let Some(k) = same {
        let o = b[k];
        if (o.bone, o.damage, o.angle, o.kbg, o.wkb, o.bkb, o.size, o.offset) == (nb.bone, nb.damage, nb.angle, nb.kbg, nb.wkb, nb.bkb, nb.size, nb.offset) {
            return;
        }
    }
    INST = INST.wrapping_add(1);
    nb.inst = INST;
    if let Some(k) = same.or_else(|| b.iter().position(|x| !x.on)) {
        b[k] = nb;
        if log {
            let q = crate::tu::vm_boss_model::arena_model().and_then(|m| crate::tu::vm_boss_model::bone_world_m(m, p, a, nb.bone, nb.offset));
            eprintln!("vm_arena: f{frame} ship hitbox {} on: {:.1} dmg, angle {}, size {:.1}, bone {} at {:?}", nb.id, nb.damage, nb.angle, nb.size, nb.bone, q);
        }
    }
}

/// The ship's live boxes, each at its bone (world x, y), with the ship's facing: (box, x, y, facing).
pub fn live_boxes(mut f: impl FnMut(&VmBox, f32, f32, f32)) {
    let (Some(p), Some(m)) = (program(), crate::tu::vm_boss_model::arena_model()) else { return };
    if !live() {
        return;
    }
    let a = actor();
    for b in unsafe { &*core::ptr::addr_of!(BOXES) }.iter().filter(|b| b.on) {
        let (x, y) = crate::tu::vm_boss_model::bone_world_m(m, p, a, b.bone, b.offset).map_or((a.pos[0], a.pos[1]), |q| (q[0], q[1]));
        f(b, x, y, if a.lr < 0.0 { -1.0 } else { 1.0 });
    }
}

/// The deck's line set at the posed bones.
unsafe fn move_deck(p: &'static Program) {
    let (Some(d), Some(m)) = (deck(), crate::tu::vm_boss_model::arena_model()) else { return };
    let a = actor();
    let mut pts = [[0.0f32; 2]; MAX_COLL_PTS];
    let n = d.len().min(MAX_COLL_PTS);
    for k in 0..n {
        match crate::tu::vm_boss_model::bone_world_m(m, p, a, d[k], [0.0; 3]) {
            Some(q) => pts[k] = q,
            None => return,
        }
    }
    crate::tu::geno_boss_coll::set_points(&pts[..n]);
}

/// The Flyer's world box (its whole mesh for its clip, and its deck points), for the camera.
pub fn bounds() -> Option<[f32; 4]> {
    if !live() {
        return None;
    }
    let s = unsafe { &*core::ptr::addr_of!(S) };
    let a = &s.vm2;
    let mut b = crate::tu::vm_boss_model::body_box(true, a).unwrap_or([a.pos[0], a.pos[0], a.pos[1], a.pos[1]]);
    for q in &s.coll_pts[..(s.coll_n as usize).min(MAX_COLL_PTS)] {
        b = [fmin(b[0], q[0]), fmax(b[1], q[0]), fmin(b[2], q[1]), fmax(b[3], q[1])];
    }
    Some(b)
}

/// Draw the Flyer (the boss's draw pass).
pub unsafe fn draw() {
    if let Some((_, r, t)) = backdrop().filter(|_| live()) {
        crate::tu::petey_model::draw_tris(r, t, [0.0, 0.0], [0.0, 0.0, 0.0], 1.0, 0.0, false);
    }
    let (Some(p), Some(m)) = (program(), crate::tu::vm_boss_model::arena_model()) else { return };
    if live() {
        crate::tu::vm_boss_model::draw_m(m, p, actor(), false, (-1, 0));
    }
}

/// The C comparisons (semantics lint M1: no IEEE min / max).
fn fmin(a: f32, b: f32) -> f32 {
    if a < b { a } else { b }
}
fn fmax(a: f32, b: f32) -> f32 {
    if a > b { a } else { b }
}
