//! The VM bosses' real look and their animation's root movement, loaded from the owner's own extracted Brawl files
//! (the same readers and rules as `petey_model.rs` / `subspace_model.rs`; nothing from the files is committed, no
//! path is printed).
//!
//! Two uses:
//! - the draw: the boss's MDL0 posed by the clip and frame its VM selected (`Actor::motion`, the motion frame);
//! - the sim: each clip's root-bone translation, sampled per frame, is fed to the VM as `Input::root_move` (the
//!   movement Brawl's own root-motion units take from the animation), and each clip's length fills
//!   `Program::motion_len`. The drawn clip holds that translation (the actor's position carries it).
//!
//! Loading happens ahead of the match: the engine-start helper (`explore::install`) calls [`prefetch`]; the match's
//! first `petey::vm()` call waits for it ([`get`] blocks while a load is running, which it only can at the match
//! start). Every allocation is made on a helper thread (system heap, outside the rollback arena).
#![allow(clippy::missing_safety_doc)]

use crate::tu::petey_model as pm;
use std::sync::{Mutex, OnceLock};
use melee_core::ppc::{fcvt_u32, Ppc};
use melee_runtime::tu::gekko_fp::{cosf, sinf};

/// The C comparisons (semantics lint M1: no IEEE min / max).
fn fmin(a: f32, b: f32) -> f32 {
    if a < b { a } else { b }
}
fn fmax(a: f32, b: f32) -> f32 {
    if a > b { a } else { b }
}

/// (VM boss name, MDL0 name; the clips are named `<MDL0><motion>`, the idle clip gives the extent)
const MODELS: &[(&str, &str, &str)] = &[
    ("rayquaza", "EnmRayquaza", "Wait1"),
    ("ridley", "EnmRidley", "Wait1"),
    ("galleom", "EnmGalleom", "Wait"),
    ("duon", "EnmDuon", "GunWait"),
    ("porky", "EnmPorky", "Wait"),
    ("metaridley", "EnmMetaridley", "Wait1"),
    ("tabuu", "EnmTaboo", "Wait"),
    // the grab-define actors (geno_boss_grab) are drawn from their define's motion; Bucculus is an SSE enemy
    ("bucculus", "EnmBucyulus", "Wait"),
];

pub struct Model {
    pub rig: pm::Rig,
    /// per VM motion: the clip index in `rig.anims` (None: the model has no clip of that name)
    pub clip: Vec<Option<usize>>,
    /// per VM motion: the root bone's (forward, up) model-space position per frame 0..=len (forward = the model's z,
    /// the axis it faces; empty: no clip / no root track)
    pub root: Vec<Vec<[f32; 2]>>,
    pub bufs: Mutex<pm::DrawBufs>,
    /// the pose buffer for bone lookups in the sim (made on the loader's thread), tagged with the (clip, frame) it
    /// holds (`Rig::pose_keyed`): the lookups of one frame share one pose
    pub sim: Mutex<pm::DrawBufs>,
    pub note: String,
}

static M: OnceLock<Option<Model>> = OnceLock::new();
static NOTE_SHOWN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn off() -> bool {
    std::env::var_os("MELEE_VMBOSS_NO_MODEL").is_some() || std::env::var_os("MELEE_BRAWL_ABSENT").is_some()
}

/// Load `name`'s model (blocking; for the engine-start helper thread). The program's motion lengths are set from the
/// clips here, so they are in place before the match takes the program.
pub fn prefetch(name: &str) {
    let _ = load(name);
}

/// The model once loaded (waits for a running load; loads now if nobody started it). None: not a VM boss with a model,
/// the files are absent, or the knob is set.
pub fn get(name: &str) -> Option<&'static Model> {
    let m = load(name);
    if let Some(m) = m {
        if !NOTE_SHOWN.swap(true, std::sync::atomic::Ordering::Relaxed) {
            eprintln!("{}", m.note);
        }
    }
    m
}

fn load(name: &str) -> Option<&'static Model> {
    if off() {
        return None;
    }
    M.get_or_init(|| {
        let n = name.to_string();
        std::thread::spawn(move || read(&n)).join().ok().flatten()
    })
    .as_ref()
}

fn read(name: &str) -> Option<Model> {
    let &(_, mdl, idle) = MODELS.iter().find(|m| m.0 == name)?;
    let &(_, _, arc, _) = brawl_boss::vm::FILES.iter().find(|f| f.0 == name)?;
    // the VM program names the motions the root tracks follow; a define-path actor (no VM run) keeps none
    let vm_run = crate::tu::petey::vm_boss(name);
    let p = if vm_run { Some(crate::tu::geno_vm::program(name)?) } else { None };
    let names: Vec<String> = p.map(|p| p.motion_names.clone()).unwrap_or_default();
    let mut rig = pm::load_rigs_with(&["stage", "adventure", arc], &[(mdl, mdl, idle)], false).ok()?.into_iter().next()??;
    // the travel carrier: the TransN (else TopN) bone nearest the root
    let tops: Vec<(String, usize)> = rig.bone_depths().into_iter().filter(|(_, d)| *d <= 2).collect();
    let pick = |suf: &str| tops.iter().filter(|(n, _)| n.ends_with(suf)).min_by_key(|(_, d)| *d).map(|(n, _)| n.clone());
    let bone = pick("TransN").or_else(|| pick("TopN"));
    let mut clip = Vec::with_capacity(names.len());
    let mut root = Vec::with_capacity(names.len());
    let mut lens: Vec<(String, u16)> = Vec::new();
    let (mut with, mut moving) = (0usize, 0usize);
    for mn in &names {
        let suf = mn.strip_prefix(mdl).unwrap_or(mn);
        let ci = rig.anims.iter().position(|(n, _)| n == suf);
        let mut track = Vec::new();
        if let Some(ci) = ci {
            with += 1;
            let a = &rig.anims[ci].1;
            let frames = fcvt_u32(fmax(a.frames, 1.0)) as u16;
            lens.push((mn.clone(), frames));
            if let Some(b) = &bone {
                let mut f = 0.0;
                while f <= a.frames {
                    if let Some(t) = a.trans_at(b, f) {
                        track.push([t[2], t[1]]);
                    }
                    f += 1.0;
                }
                let (lo, hi) = track.iter().fold((f32::MAX, f32::MIN), |(l, h), v| (fmin(l, v[0]), fmax(h, v[0])));
                if hi - lo > 0.5 {
                    moving += 1;
                }
            }
        }
        clip.push(ci);
        root.push(track);
    }
    // the drawn clips hold the travel (the actor's position carries it)
    if let Some(b) = &bone {
        for (_, a) in rig.anims.iter_mut() {
            a.hold_translation(b, [true, true, true]);
        }
    }
    if vm_run {
        crate::tu::geno_vm::set_motion_lengths(name, &lens);
    }
    let bufs = Mutex::new(rig.draw_bufs());
    let sim = Mutex::new(rig.sim_bufs());
    let note = format!(
        "vm_boss_model: {name}: model loaded ({} clips, {with} of {} VM motions have one, {moving} carry root travel on {})",
        rig.anims.len(),
        names.len(),
        bone.as_deref().unwrap_or("no root bone")
    );
    Some(Model { rig, clip, root, bufs, sim, note })
}

/// The frame of the actor's motion `el` frames after it began (the VM's own motion-frame rule).
fn frame_of(p: &brawl_boss::vm::Program, m: usize, el: u32) -> (usize, usize) {
    let len = p.motion_len.get(m).copied().unwrap_or(60).max(1) as usize;
    let el = el as usize;
    if p.motion_loop.get(m).copied().unwrap_or(false) {
        (el.prem(len), len)
    } else {
        (el.min(len - 1), len)
    }
}

/// The root move of motion `mi` for the step `el` frames after it began: the clip's root translation from that frame
/// to the next (0 across a loop's wrap, past a clip's end, and with no clip).
fn root_delta(p: &brawl_boss::vm::Program, m: &Model, mi: i32, el: u32) -> [f32; 2] {
    if mi < 0 {
        return [0.0, 0.0];
    }
    let mi = mi as usize;
    let Some(t) = m.root.get(mi).filter(|t| !t.is_empty()) else { return [0.0, 0.0] };
    let (f, len) = frame_of(p, mi, el);
    if f + 1 >= len || f + 1 >= t.len() {
        return [0.0, 0.0];
    }
    [t[f + 1][0] - t[f][0], t[f + 1][1] - t[f][1]]
}

/// The root move for the VM step about to run (the actor's motion and frame).
pub fn root_move(p: &brawl_boss::vm::Program, a: &brawl_boss::vm::Actor) -> Option<[f32; 2]> {
    let m = M.get()?.as_ref()?;
    Some(root_delta(p, m, a.motion, a.tick.wrapping_sub(a.motion_t0)))
}

/// The second motion channel's root move (kind 24): motion `mi`, begun at tick `t0` (the forwarded 040A).
pub fn root_move2(p: &brawl_boss::vm::Program, a: &brawl_boss::vm::Actor, mi: i32, t0: u32) -> [f32; 2] {
    match M.get() {
        Some(Some(m)) => root_delta(p, m, mi, a.tick.wrapping_sub(t0)),
        _ => [0.0, 0.0],
    }
}

/// The model's rotation in the world: the facing (a yaw of +-90 degrees: the model faces its z) after the rotation
/// units' turn (Actor::rot, degrees about x, y, z; applied z, y, x) [L: the posture module's order is not read].
fn model_mat(a: &brawl_boss::vm::Actor) -> [[f32; 3]; 3] {
    // the MSL sinf / cosf (gekko_fp), not the host's
    let sc = |d: f32| (sinf(d.to_radians()), cosf(d.to_radians()));
    let (sx, cx) = sc(a.rot[0]);
    let (sy, cy) = sc(a.rot[1]);
    let (sz, cz) = sc(a.rot[2]);
    let r = [
        [cy * cz, sx * sy * cz - cx * sz, cx * sy * cz + sx * sz],
        [cy * sz, sx * sy * sz + cx * cz, cx * sy * sz - sx * cz],
        [-sy, sx * cy, cx * cy],
    ];
    let s = if a.lr < 0.0 { -1.0 } else { 1.0 };
    let yaw = [[0.0, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, 0.0]];
    let mut o = [[0.0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            o[i][j] = (0..3).map(|k| yaw[i][k] * r[k][j]).sum();
        }
    }
    o
}

/// The sim's pose buffer posed at the actor's current clip frame; posed again only when the (clip, frame) changed.
fn posed<'m>(p: &brawl_boss::vm::Program, m: &'m Model, a: &brawl_boss::vm::Actor) -> Option<std::sync::MutexGuard<'m, pm::DrawBufs>> {
    let mut g = m.sim.lock().ok()?;
    let mi = a.motion.max(0) as usize;
    let clip = m.clip.get(mi).copied().flatten();
    let (f, _) = frame_of(p, mi, a.tick.wrapping_sub(a.motion_t0));
    let key = [clip.map_or(u32::MAX, |c| c as u32), f as u32];
    m.rig.pose_keyed(clip.map(|c| &m.rig.anims[c].1), f as f32, key, &mut g);
    Some(g)
}

/// Bone `bone` (its MDL0 index, as the scripts name bones) at the actor's current clip frame: a point in its local
/// space, in the world (x, y). None without the model or the bone. The joint -> node lookup for attack boxes and
/// weapon spawns; allocates nothing (the pose buffer is the loader's).
pub fn bone_world(p: &brawl_boss::vm::Program, a: &brawl_boss::vm::Actor, bone: i32, local: [f32; 3]) -> Option<[f32; 2]> {
    let m = M.get()?.as_ref()?;
    let slot = m.rig.bone_by_index(u32::try_from(bone).ok()?)?;
    let g = posed(p, m, a)?;
    let b = &*g;
    let q = pm::Rig::posed_point(b, slot, local);
    let r = model_mat(a);
    Some([a.pos[0] + r[0][0] * q[0] + r[0][1] * q[1] + r[0][2] * q[2], a.pos[1] + r[1][0] * q[0] + r[1][1] * q[1] + r[1][2] * q[2]])
}

/// Bone `bone`'s world rotation (rows; column n is its axis n) at the actor's current clip frame: the model's turn
/// times the posed bone's rotation (getNodeGlobalMtx's 3x3 part). None without the model or the bone.
pub fn bone_rot(p: &brawl_boss::vm::Program, a: &brawl_boss::vm::Actor, bone: i32) -> Option<[[f32; 3]; 3]> {
    let m = M.get()?.as_ref()?;
    let slot = m.rig.bone_by_index(u32::try_from(bone).ok()?)?;
    let g = posed(p, m, a)?;
    let b = &*g;
    let q = pm::Rig::posed_rot(b, slot)?;
    let r = model_mat(a);
    Some(core::array::from_fn(|i| core::array::from_fn(|j| (0..3).map(|k| r[i][k] * q[k][j]).sum())))
}

/// Draw the boss: its clip at the actor's motion frame, the model's origin at the actor's position, turned by its
/// facing and rotation (the same transform as [`bone_world`]).
pub unsafe fn draw(p: &brawl_boss::vm::Program, a: &brawl_boss::vm::Actor, flash: bool) -> bool {
    let Some(Some(m)) = M.get() else { return false };
    let Ok(mut b) = m.bufs.lock() else { return false };
    let mi = a.motion.max(0) as usize;
    let anim = m.clip.get(mi).copied().flatten().map(|c| &m.rig.anims[c].1);
    let (f, _) = frame_of(p, mi, a.tick.wrapping_sub(a.motion_t0));
    m.rig.skin_with(anim, f as f32, &mut b);
    pm::draw_tris_m(&m.rig, &b.tris, [0.0, 0.0], [a.pos[0], a.pos[1], 0.0], 1.0, model_mat(a), flash);
    true
}

/// Draw a define-path actor (Tabuu, Bucculus: geno_boss_grab) by its define's motion name and frame, at `x` on the
/// ground, facing `lr`. Presentation only. False until the model has loaded (or without one).
pub unsafe fn draw_named(motion: &str, frame: i32, x: f32, lr: f32, flash: bool) -> bool {
    let Some(Some(m)) = M.get() else { return false };
    if !NOTE_SHOWN.swap(true, std::sync::atomic::Ordering::Relaxed) {
        eprintln!("{}", m.note);
    }
    let Ok(mut b) = m.bufs.lock() else { return false };
    let anim = m.rig.anim(motion);
    m.rig.skin_with(anim, frame.max(0) as f32, &mut b);
    let s = if lr < 0.0 { -1.0 } else { 1.0 };
    pm::draw_tris_m(&m.rig, &b.tris, [0.0, 0.0], [x, 0.0, 0.0], 1.0, [[0.0, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, 0.0]], flash);
    true
}
