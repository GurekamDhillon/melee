//! Structural, runner, fuzz and determinism tests on a synthetic boss file built here (no real file bytes).

mod common;
use brawl_boss::archive;
use brawl_boss::file::BossFile;
use brawl_boss::fx::Fx;
use brawl_boss::rng::{FixedRng, TestRng};
use brawl_boss::runner::{AsyncWaitInLoop, Config, FrameInput, Runner, WeightRounding};
use brawl_boss::script::{Op, Target};
use brawl_boss::world::{Builtin, DamageCheck, Event, Obj, Recorder};
use common::*;

const M_WAIT: i32 = 0;
const M_ATTACK: i32 = 1;
const M_DOWN: i32 = 2;

fn hitbox_args() -> Vec<SArg> {
    let mut a = vec![
        int(0),
        int(0),
        int(9),
        int(10),
        sc(2.7),
        int(145),
        int(44),
        sc(0.0),
        int(0),
        sc(0.0),
        int(105),
        sc(-2.0),
        sc(8.0),
        sc(0.0),
        sc(0.0),
        sc(5.5),
        sc(0.0),
        int(0),
        sc(0.0),
        sc(1.0),
        sc(1.0),
    ];
    let flags = [0, 1, 20, 2, 2, 3, 0, 0, 311, 0, 15, 0, 1, 0, 0, 60, 0, 0, 3, 0, 0, 0, 0, 0, 0];
    for f in flags {
        a.push(int(f));
    }
    a.push(var(1, 1, 12));
    a
}

/// A small boss: 3 motions, statuses 2 (wait) and 4 (attack), 4 rules, a weighted wait pick and a repeat-cut list.
pub fn synth() -> Vec<u8> {
    let mut s = Synth::default();
    // general block (0x00) and tuning block (0xB0)
    s.zeros(0xB0);
    s.setf(0x08, 0.75);
    s.setf(0x9C, 10.0);
    let tuning = s.zeros(0xB0);
    s.setf(tuning, 99999.0);
    for (k, w) in [25.0f32, 10.0, 20.0, 8.0].iter().enumerate() {
        s.setf(tuning + 8 + 4 * k as u32, *w);
    }
    s.setf(tuning + 0x74, 0.3);
    // repeat-cut condition: valid; if side == entry then weight *= 0.3
    let cut = s.script(&[
        (0x120A, vec![var(2, 2, 4)]),
        (0x000A, vec![rq(7, false), var(1, 0, 1), int(2), var(2, 0, 6)]),
        (0x120F, vec![var(0, 0, 4000), var(2, 1, 0)]),
        (0x000F, vec![]),
    ]);
    // parameter block
    let pb = s.zeros(0x330);
    s.set(pb + 0x148, 2);
    s.set(pb + 0x14C, 3);
    s.setf(pb + 0x150, 0.3);
    for (k, v) in [100.0f32, -9.0, 150.0, -12.0].iter().enumerate() {
        s.setf(pb + 0x154 + 4 * k as u32, *v);
    }
    s.setf(pb + 0x174, 60.0);
    s.setf(pb + 0x178, -5.4);
    s.setf(pb + 0x17C, -68.0);
    s.setf(pb + 0x188, 68.0);
    for e in 0..2 {
        s.set(pb + 0x194 + 12 * e, 50);
        s.ptr_at(pb + 0x194 + 12 * e + 8, cut);
    }
    for (e, (b, sl)) in [(20, 0.0f32), (20, 0.0), (60, 2.0)].iter().enumerate() {
        s.set(pb + 0x1AC + 12 * e as u32, *b);
        s.setf(pb + 0x1AC + 12 * e as u32 + 4, *sl);
    }
    for (e, b) in [75u32, 25].iter().enumerate() {
        s.set(pb + 0x1E8 + 12 * e as u32, *b);
    }
    s.set(pb + 0x200, 999);
    s.set(pb + 0x27C, 999);
    s.set(pb + 0x27C + 4, 1);
    s.set(pb + 0x2F8 + 8, 50);
    s.set(pb + 0x2F8 + 12, 25);
    // motions
    let names: Vec<u32> = ["Wait", "Attack", "Down"].iter().map(|n| s.cstr(n)).collect();
    let mt = s.here();
    for (k, n) in names.iter().enumerate() {
        s.u32(if k == 0 { 0x0002_0000 } else { 0 });
        s.ptr(*n);
    }
    s.u32(0);
    // timelines
    let atk0 = s.script(&[
        (0x0001, vec![sc(4.0)]),
        (0x0004, vec![int(3)]),
        (0x1A00, vec![int(1)]),
        (0x0001, vec![sc(8.0)]),
        (0x0005, vec![]),
        (0x0002, vec![sc(40.0)]),
        (0x0619, hitbox_args()),
        (0x0A00, vec![int(9593)]),
        (0x0002, vec![sc(45.0)]),
        (0x0604, vec![]),
    ]);
    let atk1 = s.script(&[(0x0004, vec![int(3)]), (0x1A00, vec![int(0)]), (0x0002, vec![sc(6.0)]), (0x0005, vec![])]);
    let down0 = s.script(&[(0x0007, vec![imp("gameAnimCmd_CommonBossDown")])]);
    let tl = |s: &mut Synth, row: [Option<u32>; 3]| {
        let at = s.here();
        for r in row {
            match r {
                Some(p) => {
                    s.ptr(p);
                }
                None => {
                    s.u32(0);
                }
            }
        }
        at
    };
    let game0 = tl(&mut s, [None, Some(atk0), Some(down0)]);
    let game1 = tl(&mut s, [None, Some(atk1), None]);
    let empty = tl(&mut s, [None, None, None]);
    // the wait-motion pick (44011) and the status scripts
    let pick = s.script(&[
        (0x0010, vec![int(1000), var(0, 0, 44011), int(2)]),
        (0x0011, vec![int(0)]),
        (0x0409, vec![int(M_WAIT)]),
        (0x0018, vec![]),
        (0x0011, vec![int(1)]),
        (0x0409, vec![int(M_WAIT)]),
        (0x0018, vec![]),
        (0x0013, vec![]),
        (0x0008, vec![]),
    ]);
    let enter2 = s.script(&[
        (0x6A02, vec![]),
        (0x6A00, vec![int(1)]),
        (0x0007, vec![SArg::Ptr(pick)]),
        (0x6507, vec![var(0, 0, 44003), var(1, 1, 2)]),
    ]);
    let loop2 = s.script(&[
        (0x0004, vec![int(-1)]),
        (0x000A, vec![rq(2, false)]),
        (0x0007, vec![SArg::Ptr(pick)]),
        (0x000F, vec![]),
        (0x0001, vec![sc(1.0)]),
        (0x0005, vec![]),
    ]);
    let enter4 = s.script(&[(0x6A02, vec![]), (0x6A00, vec![int(3)]), (0x0409, vec![int(M_ATTACK)])]);
    let enter5 = s.script(&[(0x6A02, vec![]), (0x0409, vec![int(M_DOWN)])]);
    let st = |s: &mut Synth, items: &[(usize, Option<u32>, Option<&str>)]| {
        let at = s.zeros(17 * 4);
        for (k, p, i) in items {
            if let Some(p) = p {
                s.ptr_at(at + 4 * *k as u32, *p);
            }
            if let Some(n) = i {
                s.import_at(at + 4 * *k as u32, n);
            }
        }
        at
    };
    let loop_t = st(&mut s, &[(2, Some(loop2), None), (16, None, Some("statusAnimCmd_CommonStandBy"))]);
    let exit_t = st(&mut s, &[(16, None, Some("statusAnimCmdExit_CommonStandBy"))]);
    let enter_t = st(
        &mut s,
        &[(2, Some(enter2), None), (4, Some(enter4), None), (5, Some(enter5), None), (16, None, Some("statusAnimCmdInit_CommonStandBy"))],
    );
    // rules
    let fwd = s.script(&[(0x0007, vec![imp("statusAnimCmd_CommonNoneFlyDamageProc")])]);
    let timeout = s.script(&[
        (0x000A, vec![rq(7, false), var(0, 0, 21002), int(4), var(1, 1, 2)]),
        (0x120A, vec![var(2, 2, 4)]),
        (0x000F, vec![]),
    ]);
    let present = s.script(&[(0x000A, vec![rq(18, false), int(0), var(2, 0, 0)]), (0x120A, vec![var(2, 2, 4)]), (0x000F, vec![])]);
    let lock = s.script(&[(0x6900, vec![var(0, 0, 21004)])]);
    let rule = |s: &mut Synth, prio: i32, mask: u32, cond: Option<SArg>, action: Option<u32>, target: i32, on: bool, lk: bool| {
        let at = s.here();
        s.u32(prio as u32);
        s.u32(mask);
        match cond {
            Some(SArg::Ptr(p)) => {
                s.ptr(p);
            }
            Some(SArg::Import(n)) => {
                let f = s.u32(0);
                s.import_at(f, &n);
            }
            _ => {
                s.u32(0);
            }
        }
        match action {
            Some(p) => {
                s.ptr(p);
            }
            None => {
                s.u32(0);
            }
        }
        s.u32(target as u32);
        s.u32((on as u32) << 24 | (lk as u32) << 16);
        at
    };
    let r0 = rule(&mut s, -200, 0x10, Some(imp("statusAnimCmd_CommonAICheckDamageGround")), Some(fwd), -1, true, false);
    let r1 = rule(&mut s, 200, 0x10, Some(SArg::Ptr(timeout)), None, 4, false, false);
    let r2 = rule(&mut s, 100, 0x0F, Some(SArg::Ptr(present)), Some(lock), -1, false, true);
    let r3 = rule(&mut s, 200, 0x10, Some(imp("statusAnimCmd_CommonAICheckMotionEnd")), None, 2, false, false);
    let rt = s.here();
    for r in [r0, r1, r2, r3] {
        s.ptr(r);
    }
    s.u32(0);
    // init: side = 0, jumps wanted = 1; a cage; its L
    let init = s.script(&[
        (0x1200, vec![int(0), var(1, 0, 1)]),
        (0x1200, vec![int(1), var(1, 0, 17)]),
        (0x650F, vec![var(0, 0, 44012), int(0)]),
        (0x6E00, vec![int(0), int(5112), var(1, 1, 12)]),
        (0x000A, vec![rq(7, false), var(0, 0, 24000), int(4), sc(1.0)]),
        (0x1200, vec![int(99), var(1, 0, 1)]),
        (0x000F, vec![]),
    ]);
    // the body hurtbox and the marker root
    let hb = s.here();
    for v in [0.0f32, 0.0, 10.0, 0.0, 0.0, 10.0, 39.0] {
        s.f32(v);
    }
    s.u32(28 << 23);
    let hds = s.ptr(hb);
    s.u32(1);
    let arm = s.ptr(hb);
    s.u32(1);
    // the boss record
    let rec = s.zeros(28 * 4);
    s.ptr_at(rec, mt);
    s.ptr_at(rec + 7 * 4, tuning);
    s.ptr_at(rec + 8 * 4, pb);
    for c in 0..3u32 {
        for t in 0..3u32 {
            let tab = match (c, t) {
                (0, 0) => game0,
                (0, 1) => game1,
                _ => empty,
            };
            s.ptr_at(rec + (10 + 3 * c + t) * 4, tab);
        }
    }
    s.ptr_at(rec + 19 * 4, loop_t);
    s.ptr_at(rec + 20 * 4, exit_t);
    s.ptr_at(rec + 21 * 4, enter_t);
    s.ptr_at(rec + 22 * 4, rt);
    s.ptr_at(rec + 24 * 4, init);
    s.roots.push(("data".into(), rec));
    s.roots.push(("HitDataSet".into(), hds));
    s.roots.push(("wpnHitDataSet_ArmLeft".into(), arm));
    s.build()
}

fn rec() -> Recorder {
    let mut w = Recorder::default();
    w.lengths = vec![20, 60, 600];
    w.present = [true, false, true, false];
    w.dx = [Fx::from_int(-30), Fx::ZERO, Fx::from_int(40), Fx::ZERO];
    w
}

fn events<'a>(w: &'a Recorder, f: impl Fn(&Event) -> bool + 'a) -> impl Iterator<Item = (u32, i32, Obj, &'a Event)> + 'a {
    w.log.iter().filter(move |e| f(&e.2)).map(|e| (e.0.tick, e.0.frame, e.1, &e.2))
}

#[test]
fn decodes_structure() {
    let bytes = synth();
    let bf = BossFile::parse(&bytes).unwrap();
    assert_eq!(bf.motions.len(), 3);
    assert_eq!(bf.motions[1].name, "Attack");
    assert!(bf.motions[0].looping());
    assert_eq!(bf.rules.len(), 4);
    assert_eq!(bf.rules[2].target_mask, 0x0F);
    assert!(bf.rules[2].lock && !bf.rules[2].default_enabled && bf.rules[0].default_enabled);
    assert_eq!(bf.rules[1].target_status, 4);
    assert!(matches!(bf.rules[0].condition, Target::Import(_)));
    assert_eq!(bf.timelines[0][0].len(), 3);
    assert_eq!(bf.timelines[0][0][0], Target::None);
    assert!(matches!(bf.timelines[0][0][1], Target::Local(_)));
    assert!(matches!(bf.status_loop[16], Target::Import(_)));
    assert_eq!(bf.imports.len(), 7);
    assert_eq!(bf.body_hp(), Fx::from_int(99999));
    assert_eq!(bf.slot_repeat_factor(), Fx::ratio(3, 10));
    assert_eq!(bf.death_threshold(), Fx::from_int(10));
    assert_eq!(bf.params.int(24001), Some(2));
    assert_eq!(bf.params.scalar(4000), Some(Fx::ratio(3, 10)));
    assert_eq!(bf.params.range(44003).unwrap()[3], Fx::from_int(-12));
    let w = bf.params.weighted(44009, 3).unwrap();
    assert_eq!((w[2].base, w[2].slope), (60, Fx::from_int(2)));
    assert!(matches!(bf.params.weighted(44008, 2).unwrap()[0].condition, Target::Local(_)));
    assert!(bf.params.weighted(44008, 3).is_err());
    assert_eq!(bf.params.cage_hp(1), Some(999));
    assert_eq!(bf.params.cage_proc(0).unwrap().phase1_pct, 50);
    assert_eq!(bf.body_hurtboxes[0].bone(), 28);
    assert_eq!(bf.body_hurtboxes[0].radius, Fx::from_int(39));
    // every command typed (none unknown) and every script reachable
    for s in bf.scripts.values() {
        for c in &s.cmds {
            assert_ne!(c.op, Op::Unknown, "{:08x}", c.word);
        }
    }
    let hit = bf.scripts.values().flat_map(|s| &s.cmds).find(|c| c.id() == 0x0619).unwrap();
    assert_eq!(hit.args.len(), 47);
}

#[test]
fn unknown_commands_are_kept() {
    let bytes = synth();
    let bf = BossFile::parse(&bytes).unwrap();
    // round-trip: every argument converts back to its stored pair
    for s in bf.scripts.values() {
        for c in &s.cmds {
            for a in &c.args {
                let (t, v) = a.to_pair();
                if t != 2 {
                    assert_eq!(brawl_boss::script::Arg::from_pair(t, v), *a);
                }
            }
        }
    }
    let c = brawl_boss::script::Command::decode(0, 0x7F42_0200, vec![brawl_boss::script::Arg::Int(5)]);
    assert_eq!(c.op, Op::Unknown);
    assert_eq!(c.args.len(), 1);
    // a known id with the wrong shape is kept as unknown too
    let c = brawl_boss::script::Command::decode(0, 0x0001_0200, vec![brawl_boss::script::Arg::Int(1), brawl_boss::script::Arg::Int(2)]);
    assert_eq!(c.op, Op::Unknown);
}

#[test]
fn found_inside_nested_compressed_archives() {
    let f = synth();
    let inner = archive::build::arc(&[(1, 0, &f)]);
    let mid = archive::build::arc(&[(1, 92, &inner), (1, 93, b"models")]);
    let comp = archive::build::lz77(&mid);
    let outer = archive::build::arc(&[(1, 0, &comp), (1, 1, b"stage")]);
    assert_eq!(archive::find_petey_file(&outer).unwrap(), f);
    let other = archive::build::arc(&[(1, 0, b"nothing here")]);
    assert_eq!(archive::find_petey_file(&other), Err(brawl_boss::Error::NotFound));
}

fn run_frames(bytes: &[u8], cfg: Config, frames: u32, rng: &mut dyn brawl_boss::rng::RandomSource) -> Recorder {
    let bf = BossFile::parse(bytes).unwrap();
    let mut w = rec();
    let mut r = Runner::new(&bf, cfg);
    r.start(&mut w, rng);
    let inp = FrameInput { hp_now: Fx::from_int(99999), hp_before: Fx::from_int(99999), hit_by_player: false };
    for _ in 0..frames {
        r.step(&mut w, rng, &inp);
    }
    w
}

#[test]
fn wait_timeout_then_attack_timeline() {
    let bytes = synth();
    // draw 0 -> wait-motion pick entry 0 (75/25), draw 0 -> T = lo = 100 - 9 * 2.5 = 77.5
    let mut rng = FixedRng::new(&[0]);
    let w = run_frames(&bytes, Config::default(), 200, &mut rng);
    let st: Vec<_> = events(&w, |e| matches!(e, Event::Status { .. })).map(|e| (e.0, e.3.clone())).collect();
    assert_eq!(st[0], (0, Event::Status { from: -1, to: 2 }));
    assert_eq!(st[1], (78, Event::Status { from: 2, to: 4 }), "T = 77.5: the rule fires on status frame 78");
    // Attack (60 frames) from tick 78; the motion-end rule returns to 2 on its last frame (elapsed 59)
    assert_eq!(st[2], (78 + 59, Event::Status { from: 4, to: 2 }));
    let shakes: Vec<_> = events(&w, |e| matches!(e, Event::Shake(1))).map(|e| e.1).collect();
    assert_eq!(shakes, vec![4, 12, 20]);
    // thread 1: loop 3 { shake 0; wait until 6 } with the absolute reading
    let s0: Vec<_> = events(&w, |e| matches!(e, Event::Shake(0))).map(|e| e.1).collect();
    assert_eq!(s0, vec![0, 6, 6]);
    let hb: Vec<_> = events(&w, |e| matches!(e, Event::Hitbox(_))).collect();
    assert_eq!(hb.len(), 1);
    assert_eq!(hb[0].1, 40);
    let Event::Hitbox(h) = hb[0].3 else { unreachable!() };
    assert_eq!((h.damage, h.angle, h.kbg, h.bkb, h.bone), (16, 145, 44, 100, 9)); // 10 + 2.7 * 2.5 = 16.75 -> 16
    assert_eq!(h.size, Fx::from_int(8));
    assert_eq!(h.offset[1], Fx::ratio(55, 10));
    assert_eq!((h.shield_damage(), h.hit_sound(), h.target_mask(), h.rehit()), (20, 311, 15, 60));
    let clear: Vec<_> = events(&w, |e| matches!(e, Event::ClearHitboxes)).map(|e| e.1).collect();
    assert_eq!(clear, vec![45]);
}

#[test]
fn relative_async_wait_switch() {
    let cfg = Config { async_wait_in_loop: AsyncWaitInLoop::IterationRelative, ..Config::default() };
    let w = run_frames(&synth(), cfg, 200, &mut FixedRng::new(&[0]));
    let s0: Vec<_> = events(&w, |e| matches!(e, Event::Shake(0))).map(|e| e.1).collect();
    assert_eq!(s0, vec![0, 6, 12]);
}

#[test]
fn init_and_cage_level_truncation() {
    let w = run_frames(&synth(), Config::default(), 1, &mut FixedRng::new(&[0]));
    // L 2.5 is sent to the cage as an int: 2
    let l = events(&w, |e| matches!(e, Event::LinkInt { event: 5112, .. })).next().unwrap();
    assert_eq!(*l.3, Event::LinkInt { link: 0, event: 5112, args: vec![2] });
    assert!(events(&w, |e| matches!(e, Event::SpawnCage { param: 44012, side: 0, .. })).next().is_some());
    let bf = BossFile::parse(&synth()).unwrap();
    let mut w2 = rec();
    let mut r = Runner::new(&bf, Config::default());
    r.start(&mut w2, &mut FixedRng::new(&[0]));
    assert_eq!(r.cages[0].vars.float(0), Fx::from_int(2));
    assert_eq!(r.vars.int(1), 0, "24000 = 0: the variant branch is skipped");
    assert_eq!(r.vars.int(17), 1);
    assert_eq!(r.level(), Fx::ratio(25, 10));
}

#[test]
fn boss_down_builtin() {
    let bf = BossFile::parse(&synth()).unwrap();
    let mut w = rec();
    let mut rng = FixedRng::new(&[0]);
    let mut r = Runner::new(&bf, Config::default());
    r.start(&mut w, &mut rng);
    r.force_status(&mut w, &mut rng, 5);
    let inp = FrameInput::default();
    for _ in 0..500 {
        r.step(&mut w, &mut rng, &inp);
    }
    let ev: Vec<_> = events(&w, |e| matches!(e, Event::StageEvent(_))).map(|e| (e.1, e.3.clone())).collect();
    assert_eq!(
        ev,
        vec![(0, Event::StageEvent(5)), (1, Event::StageEvent(6)), (180, Event::StageEvent(7)), (420, Event::StageEvent(8))]
    );
}

#[test]
fn damage_rule_runs_builtin() {
    let bf = BossFile::parse(&synth()).unwrap();
    let mut w = rec();
    let mut rng = FixedRng::new(&[0]);
    let mut r = Runner::new(&bf, Config::default());
    r.start(&mut w, &mut rng);
    w.damage = vec![DamageCheck::Ground];
    r.step(&mut w, &mut rng, &FrameInput::default());
    assert!(w.log.iter().any(|e| e.2 == Event::RuleFired { rule: 0, slot: None }));
    assert!(w.log.iter().any(|e| e.2 == Event::Builtin(Builtin::NoneFlyDamageProc)));
}

#[test]
fn slot_rule_draw_and_repeat_cut() {
    let bf = BossFile::parse(&synth()).unwrap();
    let mut w = rec();
    // draws: the wait pick, T; then the slot draw
    let mut rng = FixedRng::new(&[0, 0, 0]);
    let mut r = Runner::new(&bf, Config::default());
    r.start(&mut w, &mut rng);
    // enable the per-slot lock rule (priority 100 beats the 200 timeout)
    // slots 0 and 2 are present: weights 25 and 20; draw 0 -> slot 0
    r.set_rule_enabled(2, true);
    let before = rng.draws;
    r.step(&mut w, &mut rng, &FrameInput::default());
    assert_eq!(rng.draws, before + 1, "one slot draw");
    assert_eq!(r.lock, Some(0));
    assert_eq!(r.last_slot, Some(0));
    // the next draw: slot 0 is cut to 25 * 0.3 = 7.5; total 27.5. u = 0x5000_0000 -> r = 27.5 * 0.3125 = 8.59 -> slot 2
    let mut rng2 = FixedRng::new(&[0x5000_0000]);
    r.lock = None;
    r.step(&mut w, &mut rng2, &FrameInput::default());
    assert_eq!(r.lock, Some(2));
    // without the cut it would have been slot 0 (25 / 45 = 0.555 > 0.3125)
}

#[test]
fn weighted_switch_repeat_cut() {
    let bf = BossFile::parse(&synth()).unwrap();
    let list = brawl_boss::script::Var::new(0, 0, 44008);
    for (side, u, expect) in [(0, 0x3000_0000u32, 0), (0, 0x3C00_0000, 1), (1, 0x3C00_0000, 0)] {
        let mut w = rec();
        let mut rng = FixedRng::new(&[0, 0]);
        let mut r = Runner::new(&bf, Config::default());
        r.start(&mut w, &mut rng);
        r.vars.ints.insert(1, side);
        let mut rng2 = FixedRng::new(&[u]);
        let pick = r.weighted_pick(&mut w, &mut rng2, list, 2);
        assert_eq!(rng2.draws, 1);
        // side 0: weights 15, 50 (total 65): u/2^32 * 65 < 15 <=> u < 0x3B13_B13B
        assert_eq!(pick, Some(expect), "side {side} u {u:#x}");
    }
    // truncation switches give the same result on integer-valued weights
    for wr in [WeightRounding::TruncateBeforeCut, WeightRounding::TruncateAfterCut] {
        let mut w = rec();
        let mut rng = FixedRng::new(&[0]);
        let mut r = Runner::new(&bf, Config { weight_rounding: wr, ..Config::default() });
        r.start(&mut w, &mut rng);
        assert_eq!(r.weighted_pick(&mut w, &mut FixedRng::new(&[0x3000_0000]), list, 2), Some(0));
    }
}

#[test]
fn determinism() {
    let bytes = synth();
    let a = run_frames(&bytes, Config::default(), 2000, &mut TestRng(7));
    let b = run_frames(&bytes, Config::default(), 2000, &mut TestRng(7));
    assert_eq!(a.log, b.log);
    assert!(a.log.len() > 50);
}

#[test]
fn truncated_and_corrupted_input_never_panics() {
    let f = synth();
    for n in 0..f.len() {
        let _ = BossFile::parse(&f[..n]);
        let _ = archive::find_petey_file(&f[..n]);
    }
    assert!(BossFile::parse(&f[..f.len() - 1]).is_err());
    let mut rng = TestRng(1);
    use brawl_boss::rng::RandomSource;
    let mut parsed = 0;
    for _ in 0..3000 {
        let mut g = f.clone();
        let k = 1 + rng.next_u32() % 4;
        for _ in 0..k {
            let at = rng.next_u32() as usize % g.len();
            g[at] = rng.next_u32() as u8;
        }
        if let Ok(bf) = BossFile::parse(&g) {
            parsed += 1;
            let mut w = rec();
            let mut rr = TestRng(3);
            let mut r = Runner::new(&bf, Config::default());
            r.start(&mut w, &mut rr);
            for _ in 0..120 {
                r.step(&mut w, &mut rr, &FrameInput::default());
            }
        }
    }
    assert!(parsed > 0);
    // corrupted archives and LZ streams
    let arc = archive::build::arc(&[(1, 0, &archive::build::lz77(&archive::build::arc(&[(1, 0, &f)])))]);
    for n in (0..arc.len()).step_by(7) {
        let _ = archive::find_petey_file(&arc[..n]);
    }
    for _ in 0..2000 {
        let mut g = arc.clone();
        let at = rng.next_u32() as usize % g.len();
        g[at] = rng.next_u32() as u8;
        let _ = archive::find_petey_file(&g);
    }
}
