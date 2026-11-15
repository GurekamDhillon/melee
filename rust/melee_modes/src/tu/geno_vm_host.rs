//! The VS host's side of a VM-run boss (geno_vm): one frame of the enemy-command VM with the world it reads, and
//! the presentation it forwards back. Called from `petey::tick` for every boss in `brawl_boss::vm::FILES` the
//! generic host gives to the VM.
//!
//! * Input: the other fighters as targets, this frame's damage on the stand-in body, ground / floor, and the ground
//!   module's surface queries (`Input::ground_q`).
//! * Events: status / stage / remove are logged; the forwarded hitbox commands (0619 set, 0604 clear) keep the live
//!   attack boxes in [`BOXES`], which the host turns into the carrier's hit capsules.
//!
//! Plain statics, no allocation in the frame: the rollback snapshot covers them.
#![allow(static_mut_refs, clippy::missing_safety_doc)]

use brawl_boss::vm::{self as vr, Program, A};
use melee_types::*;

/// One live attack box from a 0619 (the command's own id, its numbers already at the actor's L).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VmBox {
    pub on: bool,
    /// the 0619's box id (a new 0619 with the same id replaces it)
    pub id: u8,
    pub bone: i32,
    pub damage: f32,
    pub angle: i32,
    pub kbg: f32,
    pub wkb: f32,
    pub bkb: f32,
    pub size: f32,
    pub offset: [f32; 3],
    /// bumps on every set: a new hit group for the victims
    pub inst: u32,
}

pub const MAX_BOXES: usize = 8;
pub static mut BOXES: [VmBox; MAX_BOXES] = [VmBox { on: false, id: 0, bone: 0, damage: 0.0, angle: 0, kbg: 0.0, wkb: 0.0, bkb: 0.0, size: 0.0, offset: [0.0; 3], inst: 0 }; MAX_BOXES];
static mut INST: u32 = 0;

/// Not a VM match / a new one: no boxes.
pub unsafe fn reset() {
    for b in BOXES.iter_mut() {
        b.on = false;
    }
    INST = 0;
    crate::tu::geno_boss::S.spawn = [crate::tu::geno_boss_spawn::FREE; crate::tu::geno_boss_spawn::MAX_SPAWN];
    // the boss's weapon records (local data), loaded here at the match's start, never in the frame
    WEAPONS = crate::tu::geno_boss_spawn::vm_records(crate::tu::petey::boss_name());
    SEC = (-1, 0);
    VIS_SLOT = (-1, 0);
    CAUGHT = false;
    CATCH_OFF = [0.0; 3];
    THROW = None;
    crate::tu::geno_boss::S.grab = crate::tu::geno_boss_grab::OFF;
}

/// The VM boss's weapon records (650E spawns; geno_boss_spawn), set by [`reset`].
pub static mut WEAPONS: &[crate::tu::geno_boss_spawn::SpawnRecord] = &[];

/// A 650E (EV_WEAPON): the record by its param id, a projectile in the pool at the actor's point, facing and L.
/// Speed and angle draws come from the actor's own generator (in the snapshot and the hash).
unsafe fn weapon(p: &Program, frame: u32, ev: &vr::Ev, log: bool) {
    use crate::tu::geno_boss::S;
    let Some(rec) = crate::tu::geno_boss_spawn::record_index(WEAPONS, ev.v as u32) else {
        if log {
            eprintln!("petey: f{frame} vm weapon {} has no local record", ev.v);
        }
        return;
    };
    let joint = p.code.get(ev.ins as usize).filter(|i| i.n >= 3).and_then(|i| p.args.get((i.a + 2) as usize)).map_or(-1, |&x| melee_core::ppc::fctiwz_f32(arg(&S.vm, x)) as i16);
    let mut draw = || {
        S.vm.rng = S.vm.rng.wrapping_mul(1_103_515_245).wrapping_add(12345);
        ((S.vm.rng >> 8) & 0xFFFF) as f32 / 65536.0
    };
    let (r1, r2) = (draw(), draw());
    let a = S.vm;
    let face = if a.lr < 0.0 { -1.0 } else { 1.0 };
    let slot = crate::tu::geno_boss_spawn::spawn_at(WEAPONS, &mut *core::ptr::addr_of_mut!(S), rec, joint, a.pos[0], a.pos[1], face, a.level(), r1, r2);
    // at the script's bone, posed at this frame (the record's offset in the bone's space); no model / no such bone:
    // the actor's point plus the offset, as spawned
    let mut at_bone = false;
    // (no joint in the 650E: the record's own bone, which spawn_at put in the slot)
    if let Some(i) = slot.filter(|&i| S.spawn[i].joint >= 0) {
        let off = WEAPONS[rec as usize].offset;
        if let Some([x, y]) = crate::tu::vm_boss_model::bone_world(p, crate::tu::geno_vm::actor(), S.spawn[i].joint as i32, [off[0], off[1], 0.0]) {
            crate::tu::geno_boss_spawn::place(i, x, y);
            at_bone = true;
        }
    }
    if log {
        let q = slot.map(|i| (S.spawn[i].x, S.spawn[i].y)).unwrap_or_default();
        eprintln!("petey: f{frame} vm weapon {} joint {joint} -> slot {slot:?} at ({:.1}, {:.1}){}", ev.v, q.0, q.1, if at_bone { " on its bone" } else { "" });
    }
}

/// The pool's frame: projectiles move, home, explode and end; the players are its targets (contact / homing).
unsafe fn weapons_step(carrier: i32, fighter_of: unsafe fn(i32) -> *mut Fighter, frame: u32, log: bool) {
    use crate::tu::geno_boss_spawn as sp;
    if WEAPONS.is_empty() {
        return;
    }
    let mut t: [sp::Target; 4] = [None; 4];
    for s in 0..4i32 {
        let f = fighter_of(s);
        if s != carrier && !f.is_null() {
            // the body's centre, about a fighter's half height up
            t[s as usize] = Some(((*f).cur_pos.x, (*f).cur_pos.y + 8.0, 6.0));
        }
    }
    let mut out = [(0, 0, 0.0, 0, 0.0, 0.0, 0.0); sp::MAX_SPAWN];
    let before = sp::live_count();
    let n = sp::step(WEAPONS, &t, &mut out);
    if log {
        for h in &out[..n] {
            eprintln!("petey: f{frame} vm weapon slot {} touches player {} ({:.1} dmg)", h.0, h.1, h.2);
        }
    }
    if log && sp::live_count() != before {
        eprintln!("petey: f{frame} vm weapons live {}", sp::live_count());
    }
}

/// The value of one compiled argument as the host can read it (constants and the actor's work / temp variables;
/// anything the VM computes from the engine reads as 0).
fn arg(a: &vr::Actor, x: A) -> f32 {
    match x {
        A::K(k) => k,
        A::Wi(i) => a.wi.get(i as usize).map_or(0.0, |&v| v as f32),
        A::Wf(i) => a.wf.get(i as usize).copied().unwrap_or(0.0),
        A::Ti(i) => a.ti.get(i as usize).map_or(0.0, |&v| v as f32),
        A::Tf(i) => a.tf.get(i as usize).copied().unwrap_or(0.0),
        _ => 0.0,
    }
}

/// A forwarded 0619: (id, part, bone, damage + per-L, angle, growth + per-L, set + per-L, base + per-L, size + per-L,
/// offset x y z, ...), the numbers taken at the actor's L.
unsafe fn set_box(p: &Program, a: &vr::Actor, ins: &vr::Ins) {
    if ins.n < 17 {
        return;
    }
    let v = |k: u32| p.args.get((ins.a + k) as usize).map_or(0.0, |&x| arg(a, x));
    let l = a.level();
    let lin = |k: u32| v(k) + v(k + 1) * l;
    let id = melee_core::ppc::fctiwz_f32(v(0)) as u8;
    INST = INST.wrapping_add(1);
    let nb = VmBox {
        on: true,
        id,
        bone: melee_core::ppc::fctiwz_f32(v(2)),
        damage: lin(3),
        angle: melee_core::ppc::fctiwz_f32(v(5)),
        kbg: lin(6),
        wkb: lin(8),
        bkb: lin(10),
        size: lin(12),
        offset: [v(14), v(15), v(16)],
        inst: INST,
    };
    let slot = BOXES.iter().position(|b| b.on && b.id == id).or_else(|| BOXES.iter().position(|b| !b.on));
    if let Some(s) = slot {
        BOXES[s] = nb;
    }
}

/// What the host mirrors after a VM frame.
pub struct Out {
    pub status: i32,
    pub x: f32,
    pub y: f32,
    pub lr: f32,
    pub dead: bool,
}

/// The world a VM actor reads this frame. `ground`: the carrier stands on a floor.
unsafe fn input(carrier: i32, damage: i32, ground: bool, fighter_of: unsafe fn(i32) -> *mut Fighter) -> vr::Input {
    let mut i = vr::Input { ground, ..Default::default() };
    collision(&mut i);
    for s in 0..4i32 {
        let f = fighter_of(s);
        if s != carrier && !f.is_null() {
            i.targets[s as usize] = Some([(*f).cur_pos.x, (*f).cur_pos.y]);
            // 6901 / 6902 / R10024-R10047's per-target factors: its facing and its model scale
            i.target_lr[s as usize] = (*f).facing_dir;
            i.target_scale[s as usize] = (*f).x34_scale.y;
        }
    }
    i.damage = damage as f32;
    // the ground module's surface queries (R63-R69). The IR leaves what each computes unread [L]; read here as:
    // R63 "on a surface" = standing on a floor, R64 "the surface goes on ahead" = on a floor and not at the arena's
    // edge in the facing direction; R65-R69 (surface kind / line / side tests) false. Enough for a crawler (Glunder)
    // to walk its floor and turn at the end.
    let a = crate::tu::geno_vm::actor();
    let bz = &*core::ptr::addr_of!(crate::tu::script_game::stage_info.blast_zone);
    let edge = if a.lr >= 0.0 { a.pos[0] >= bz.right * 0.5 } else { a.pos[0] <= bz.left * 0.5 };
    let mut q = 0u32;
    if i.ground {
        q |= 1;
        if !edge {
            q |= 2;
        }
    }
    i.ground_q = q;
    i.cliff = [i.ground && a.pos[0] <= bz.left * 0.5, i.ground && a.pos[0] >= bz.right * 0.5];
    i
}

/// The stage's floor under the VM actor and its nearest wall each side, from the live collision (the VM stops its
/// movement on them); a floor found also decides `ground` (the actor's feet on it). Probes start a little above the
/// actor's feet.
unsafe fn collision(i: &mut vr::Input) {
    use melee_world::tu::mplib as mp;
    let a = crate::tu::geno_vm::actor();
    let (x, y) = (a.pos[0], a.pos[1]);
    let (mut p, mut n): (Vec, Vec) = (core::mem::zeroed(), core::mem::zeroed());
    let (mut line, mut flags) = (-1i32, 0u32);
    if mp::mpCheckFloor(x, y + 10.0, x, y - 400.0, 0.0, &mut p, &mut line, &mut flags, &mut n, -1, -1, -1, None, core::ptr::null_mut()) != 0 {
        i.floor_on = true;
        i.floor_y = p.y;
        i.ground = (y - p.y).abs() <= 1.0;
    }
    let yh = y + 10.0;
    for (side, dx) in [(0usize, -400.0f32), (1, 400.0)] {
        for f in [mp::mpCheckLeftWall, mp::mpCheckRightWall] {
            if f(x, yh, x + dx, yh, &mut p, &mut line, &mut flags, &mut n, -1, -1) != 0 && (p.x - x) * dx >= 0.0 {
                if i.wall_on & (1 << side) == 0 || (p.x - x).abs() < (i.wall[side] - x).abs() {
                    i.wall[side] = p.x;
                    i.wall_on |= 1 << side;
                }
            }
        }
    }
}

/// One VM frame: input, `geno_vm::tick`, the events (logged with `log`, hitboxes kept), the actor's state out.
/// The VM's catch-exit event (the common CommonCatchProcExit routine ran in a status exit).
const EV_CATCH_EXIT: u32 = vr::EV_CATCH_EXIT;
/// The catch host's state (statics of the executable, so the snapshot carries them): a catch made last frame, the
/// catch box's offset in its bone, the throw box of a pending release.
static mut CAUGHT: bool = false;
static mut CATCH_OFF: [f32; 3] = [0.0; 3];
static mut THROW: Option<crate::tu::geno_boss_grab::Throw> = None;

/// The animation root movement fed to the VM since the last status change (log only, not hashed).
static mut ROOT_TRAVEL: f32 = 0.0;

/// The motion module's second (added / blended) motion: (motion, the tick it began), set by a forwarded 040A
/// ("040A n": blend motion n in; Tabuu's DiveStart1), cleared on a motion change. -1: none. A static of the
/// executable, so the snapshot carries it.
static mut SEC: (i32, u32) = (-1, 0);

/// The motion module's extra animation slot 0 ("0410 0 n": motion n's file played beside the body's motion; Porky's
/// SparkDspOff / GrSparkDspOn / AirSparkDspOn, whose files hold only a VIS0): (motion, the tick it began), kept until
/// the next 0410 on the slot [L: the slot outlives motion changes; its VIS0 runs on its own frame]. Drawing only
/// (vm_boss_model applies its VIS0); a static of the executable, so the snapshot carries it.
static mut VIS_SLOT: (i32, u32) = (-1, 0);

/// The extra animation slot's (motion, start tick) for the draw (-1: none).
pub fn vis_slot() -> (i32, u32) {
    unsafe { VIS_SLOT }
}

pub unsafe fn frame(p: &'static Program, carrier: i32, frame: u32, damage: i32, ground: bool, fighter_of: unsafe fn(i32) -> *mut Fighter, log: bool) -> Out {
    let mut i = input(carrier, damage, ground, fighter_of);
    // out of the boss area (the program's box, the stage's camera bounds): the AreaOut rule (-300) turns the boss back
    {
        let a = crate::tu::geno_vm::actor();
        let b = p.bounds;
        i.out_of_area = a.pos[0] <= b[0] || a.pos[0] >= b[1] || a.pos[1] <= b[2] || a.pos[1] >= b[3];
    }
    // the animation's root movement for this step (the clip and frame the VM selected), when the boss has its model
    if let Some(mv) = crate::tu::vm_boss_model::root_move(p, crate::tu::geno_vm::actor()) {
        i.root_on = true;
        i.root_move = mv;
        // the model is drawn at scale 1 (petey::MODEL_SCALE); kind 24 moves by the second motion channel (SEC)
        i.root_scale = 1.0;
        if SEC.0 >= 0 {
            i.root_move2 = crate::tu::vm_boss_model::root_move2(p, crate::tu::geno_vm::actor(), SEC.0, SEC.1);
        }
        ROOT_TRAVEL += mv[0].abs() + mv[1].abs();
    }
    model_nodes(p, &mut i);
    // the catch host's last frame: a catch (the VM sets its caught flag) and the hold
    i.caught = CAUGHT;
    i.holding = crate::tu::geno_boss::S.grab.victim >= 0;
    CAUGHT = false;
    let last = crate::tu::geno_vm::actor().status;
    crate::tu::geno_vm::tick(p, &i);
    let a = crate::tu::geno_vm::actor();
    let mut wev = [vr::Ev { kind: 0, ins: 0, v: 0 }; 8];
    let mut nw = 0;
    let mut release = false;
    let mut box_now: Option<VmBox> = None;
    for e in a.events() {
        match e.kind {
            EV_CATCH_EXIT => {
                if log {
                    eprintln!("petey: f{frame} vm catch exit (CommonCatchProcExit) in status {}", a.status);
                }
                // CommonCatchProcExit: the hold ends with this status (a throw already asked for goes first)
                let g = &mut *core::ptr::addr_of_mut!(crate::tu::geno_boss::S.grab);
                if g.victim >= 0 && g.pending == 0 {
                    g.pending = crate::tu::geno_boss_grab::GRAB_DROP;
                    g.throw = -1;
                }
            }
            vr::EV_STATUS => {
                // the catch box belongs to the status that armed it
                let g = &mut *core::ptr::addr_of_mut!(crate::tu::geno_boss::S.grab);
                if g.armed && g.status != e.v {
                    g.armed = false;
                }
                if log {
                    eprintln!("petey: f{frame} status {last} -> {} px {:.1} hp {:.1}", e.v, a.pos[0], a.hp());
                    if ROOT_TRAVEL > 0.0 {
                        eprintln!("petey: f{frame} vm root travel {:.1} in status {last} (py {:.1})", ROOT_TRAVEL, a.pos[1]);
                    }
                    ROOT_TRAVEL = 0.0;
                }
                // a status change ends the motion's boxes
                reset_boxes();
            }
            vr::EV_MOTION => {
                reset_boxes();
                SEC = (-1, 0);
            }
            vr::EV_WEAPON => {
                // spawned after the event loop (the spawn draws from the actor's generator)
                if nw < wev.len() {
                    wev[nw] = *e;
                    nw += 1;
                }
            }
            vr::EV_STAGE => eprintln!("petey: f{frame} stage event {}", e.v),
            vr::EV_REMOVE => eprintln!("petey: f{frame} vm actor removed itself"),
            vr::EV_NOTE => eprintln!(
                "petey: f{frame} stand-in: goal {:.1} beyond the arena on axis {}, arrival taken at the limit",
                a.goal[e.ins as usize & 1],
                e.ins
            ),
            vr::EV_CMD => {
                let Some(ins) = p.code.get(e.ins as usize) else { continue };
                match ins.id {
                    0x040A => {
                        let m = p.args.get(ins.a as usize).map_or(-1.0, |&x| arg(a, x));
                        SEC = (melee_core::ppc::fctiwz_f32(m), a.tick.wrapping_sub(1));
                    }
                    0x0410 if ins.n >= 2 => {
                        let v = |k: u32| p.args.get((ins.a + k) as usize).map_or(-1.0, |&x| arg(a, x));
                        // the same motion again keeps running (Porky's scripts reissue it every other frame) [L]
                        let m = melee_core::ppc::fctiwz_f32(v(1));
                        if melee_core::ppc::fctiwz_f32(v(0)) == 0 && VIS_SLOT.0 != m {
                            VIS_SLOT = (m, a.tick.wrapping_sub(1));
                            if log {
                                eprintln!("petey: f{frame} vm vis slot: motion {}", VIS_SLOT.0);
                            }
                        }
                    }
                    // the catch: 060A (slot, bone, size, offset x y z, ...) arms a catch box, 060D ends it, 061A
                    // defines the throw (slot, ?, damage + per-L, angle, growth, weight-set, -, -, base [I: the
                    // define path's reading]), 6528 releases the held fighter with that frame's throw
                    0x060A => {
                        let v = |k: u32| p.args.get((ins.a + k) as usize).map_or(0.0, |&x| arg(a, x));
                        let g = &mut *core::ptr::addr_of_mut!(crate::tu::geno_boss::S.grab);
                        crate::tu::geno_boss_grab::arm(g, 0, melee_core::ppc::fctiwz_f32(v(1)), v(2), (0, i32::MAX), !0, a.status);
                        CATCH_OFF = [v(3), v(4), v(5)];
                        if log {
                            eprintln!("petey: f{frame} vm catch box on: bone {} size {:.1}", g.joint, g.radius);
                        }
                    }
                    0x060D => (*core::ptr::addr_of_mut!(crate::tu::geno_boss::S.grab)).armed = false,
                    0x061A => {
                        let v = |k: u32| p.args.get((ins.a + k) as usize).map_or(0.0, |&x| arg(a, x));
                        let z = melee_core::ppc::fctiwz_f32;
                        THROW = Some(crate::tu::geno_boss_grab::Throw { damage: v(2) + v(3) * a.level(), angle: z(v(4)), kbg: z(v(5)), wkb: z(v(6)), bkb: z(v(9)) });
                    }
                    0x6528 => {
                        release = true;
                        if log {
                            eprintln!("petey: f{frame} vm release (6528) in status {}", a.status);
                        }
                    }
                    0x0619 => {
                        set_box(p, a, ins);
                        box_now = BOXES.iter().find(|b| b.on && b.inst == INST).copied();
                        if log {
                            if let Some(b) = BOXES.iter().find(|b| b.on && b.inst == INST) {
                                eprintln!("petey: f{frame} vm hitbox {} on: {:.1} dmg, angle {}, size {:.1}", b.id, b.damage, b.angle, b.size);
                                if let Some(q) = crate::tu::vm_boss_model::bone_world(p, a, b.bone, b.offset) {
                                    eprintln!(
                                        "petey: f{frame} vm hitbox {} on bone {} at {:.1} {:.1} (actor {:.1} {:.1})",
                                        b.id, b.bone, q[0], q[1], a.pos[0], a.pos[1]
                                    );
                                }
                            }
                        }
                    }
                    0x0604 => reset_boxes(),
                    _ => {}
                }
            }
            _ => {}
        }
    }
    for e in &wev[..nw] {
        weapon(p, frame, e, log);
    }
    weapons_step(carrier, fighter_of, frame, log);
    let a = crate::tu::geno_vm::actor();
    // a release: the throw this frame defined (061A), else the attack box it opened (Tabuu's ChainAttack)
    if release {
        let g = &mut *core::ptr::addr_of_mut!(crate::tu::geno_boss::S.grab);
        let t = THROW.take().or_else(|| {
            box_now.map(|b| {
                let z = melee_core::ppc::fctiwz_f32;
                crate::tu::geno_boss_grab::Throw { damage: b.damage, angle: b.angle, kbg: z(b.kbg), wkb: z(b.wkb), bkb: z(b.bkb) }
            })
        });
        THROW = t;
        crate::tu::geno_boss_grab::request_release(g, t.map(|_| 0));
    }
    let g = &*core::ptr::addr_of!(crate::tu::geno_boss::S.grab);
    let catch = (g.armed && g.victim < 0).then(|| (crate::tu::vm_boss_model::bone_world(p, a, g.joint, CATCH_OFF), g.radius));
    if log && catch.is_some() {
        let t = (fighter_of)(0);
        let (tx, ty) = if t.is_null() { (0.0, 0.0) } else { ((*t).cur_pos.x, (*t).cur_pos.y) };
        eprintln!("petey: f{frame} vm catch box live at {:?} r {:.1} (actor {:.1} {:.1}, player 1 at {tx:.1} {ty:.1})", catch.and_then(|c| c.0), g.radius, a.pos[0], a.pos[1]);
    }
    let throw = THROW;
    let box_of = |i: i32| (i == 0).then_some(()).and(throw).map(|t| (t, "vm throw"));
    if crate::tu::geno_boss_grab::host_core(carrier, frame, &box_of, catch) {
        CAUGHT = true;
    }
    if crate::tu::geno_boss::S.grab.victim < 0 && crate::tu::geno_boss::S.grab.pending == 0 {
        THROW = None;
    }
    Out { status: a.status, x: a.pos[0], y: a.pos[1], lr: a.lr, dead: a.dead() || a.live == 0 }
}

/// The model's answers for a boss with a skeleton: the enemy's centre (getCenterPos: the node value 23014 names) and
/// the node rotations R10006 / R10007 / R10024-R10047 ask for (their constant node arguments, the first two found;
/// node ids are MDL0 bone indices). Without the model the VM's fallbacks stay.
fn model_nodes(p: &Program, i: &mut vr::Input) {
    let a = crate::tu::geno_vm::actor();
    if let Some(&(_, t, off)) = p.block_values.iter().find(|v| v.0 == 23014) {
        let at = (if t == 0 { 0 } else { p.tuning_at } + off) as usize;
        if let Some(w) = p.data.get(at..at + 4) {
            let node = u32::from_be_bytes([w[0], w[1], w[2], w[3]]) as i32;
            i.center = crate::tu::vm_boss_model::bone_world(p, a, node, [0.0; 3]);
        }
    }
    let mut n = 0;
    for ins in &p.code {
        if n == i.nodes.len() {
            break;
        }
        for k in 0..ins.n as u32 {
            let Some(&A::Req { id, .. }) = p.args.get((ins.a + k) as usize) else { continue };
            if !matches!(id, 10006 | 10007 | 10024..=10047) {
                continue;
            }
            let Some(&A::K(v)) = p.args.get((ins.a + k + 1) as usize) else { continue };
            let node = v as i32;
            if n < i.nodes.len() && !i.nodes[..n].iter().any(|x| x.id == node) {
                if let Some(rot) = crate::tu::vm_boss_model::bone_rot(p, a, node) {
                    i.nodes[n] = vr::NodeRot { on: true, id: node, rot };
                    n += 1;
                }
            }
        }
    }
}

unsafe fn reset_boxes() {
    for b in BOXES.iter_mut() {
        b.on = false;
    }
}
