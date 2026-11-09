//! lab-check part 7: drills, the adaptive trainer, mirror / jitter on load (lab-theorycraft.md 4b, 8 C, K).
//!
//! * every starter drill parses, and its success predicate fires on a scripted success stream and not on a scripted
//!   failure (the judge is a pure function of the frame's events, so the streams are the events a real attempt emits);
//! * the weights shift after scripted failures: on the engine, a drill whose predicate the scripted P1 cannot meet
//!   (hit within 2 frames of the attempt start) fails every attempt, and the adaptive trainer moves weight onto the
//!   options it picked; one it always meets (safe for 2 frames) drives them back to 1; twice from the same seed the run
//!   is hash-equal;
//! * mirror / jitter: a load with the variation disabled again is hash-equal to a plain load; enabled it differs, and
//!   is the same twice from the same seed.

use super::{diff, run};
use crate::lab::train::{self, JFrame, Judge, Verdict};
use crate::lab::LabRun;
use melee_modhost::lab::*;
use std::cell::RefCell;
use std::rc::Rc;

fn ev(kind: u8, port: u8, frame: i32, a: i32) -> LabEvent {
    LabEvent { kind, port, frame, a, b: 0 }
}

/// Feed a scripted attempt: (frame offset, events, dummy action, user action) per frame; the verdict, if any.
fn feed(dr: &train::Drill, frames: &[(i32, Vec<LabEvent>, i32, i32)]) -> Option<Verdict> {
    let mut j = Judge::new(dr, 1, 0, 0);
    let mut prev = 0;
    for f in 0..1000 {
        let (evs, dm, um) = frames.iter().find(|x| x.0 == f).map_or((Vec::new(), prev, 14), |x| (x.1.clone(), x.2, x.3));
        let fr = JFrame { f, ev: &evs, d_prev: prev, d_motion: dm, u_motion: um };
        prev = dm;
        if let Some(v) = j.feed(&fr) {
            return Some(v);
        }
    }
    None
}

pub fn train_checks(lab: &mut LabRun, sh: &Rc<RefCell<LabShared>>, check: &mut dyn FnMut(bool, String)) {
    let c = LabCmd::new;
    // ---- the starter drills' predicates on scripted streams
    const W: i32 = 14; // Wait: an idle action
    let hit_d = |f| ev(EV_HIT, 1, f, 0);
    let hit_u = |f| ev(EV_HIT, 0, f, 1);
    for (name, src) in train::BUILTIN {
        let dr = match train::parse(src) {
            Ok(d) => d,
            Err(e) => {
                check(false, format!("drill {name} parses: {e}"));
                continue;
            }
        };
        // (success stream, failure stream)
        let (ok, bad): (Vec<(i32, Vec<LabEvent>, i32, i32)>, Vec<(i32, Vec<LabEvent>, i32, i32)>) = match name {
            "shield_punish" => (
                vec![(5, vec![hit_d(5)], W, W), (10, vec![ev(EV_SHIELD, 0, 10, 1)], W, W), (22, vec![hit_d(22)], W, W)],
                vec![(10, vec![ev(EV_SHIELD, 0, 10, 1)], W, W), (20, vec![hit_u(20)], W, W)],
            ),
            "tech_chase" => (
                vec![(3, vec![hit_d(3)], W, W), (30, vec![ev(EV_TECH, 1, 30, 2)], 200, W), (48, vec![hit_d(48)], 75, W)],
                vec![(30, vec![ev(EV_TECH, 1, 30, 2)], 200, W)],
            ),
            "hit_confirm" => (
                vec![(10, vec![hit_d(10)], 75, W), (25, vec![hit_d(25)], 75, W)],
                vec![(10, vec![ev(EV_SHIELD, 1, 10, 0)], 181, W), (20, vec![], 179, 225)],
            ),
            "ledge" => (
                vec![(0, vec![], 253, W), (40, vec![], 254, W), (60, vec![hit_d(60)], 75, W)],
                vec![(0, vec![], 253, W), (40, vec![], 254, W), (70, vec![hit_u(70)], 254, W)],
            ),
            "oos" => (
                vec![(20, vec![ev(EV_OOS, 1, 20, 25)], 25, W), (35, vec![hit_d(35)], 75, W)],
                vec![(20, vec![ev(EV_OOS, 1, 20, 25)], 25, W), (26, vec![hit_u(26)], 25, 75)],
            ),
            _ => (Vec::new(), Vec::new()),
        };
        let (a, b) = (feed(&dr, &ok), feed(&dr, &bad));
        check(
            matches!(a, Some(Verdict::Success { .. })) && matches!(b, Some(Verdict::Missed | Verdict::Punished)),
            format!("drill `{}`: success predicate fires on a scripted success ({a:?}) and not on a scripted failure ({b:?})", dr.name),
        );
    }
    // the hit-confirm drill's other branch: shielded, the user stops and stays safe; hit, the user drops the combo
    {
        let dr = train::parse(train::BUILTIN[2].1).unwrap();
        let a = feed(&dr, &[(10, vec![ev(EV_SHIELD, 1, 10, 0)], 181, W)]);
        let b = feed(&dr, &[(10, vec![hit_d(10)], 75, W)]);
        check(matches!(a, Some(Verdict::Success { .. })) && b == Some(Verdict::Missed), format!("hit confirm, the other branches: shield then safe {a:?}; hit then no follow-up {b:?}"));
    }
    // a hit before the trigger is not a success
    {
        let dr = train::parse(train::BUILTIN[1].1).unwrap();
        let v = feed(&dr, &[(3, vec![hit_d(3)], 75, W)]);
        check(v == Some(Verdict::NoTrigger), format!("tech chase: a hit with no tech is not scored ({v:?})"));
    }
    // the update rule itself
    {
        let mut w = vec![4u8, 4, 4, 4];
        for _ in 0..5 {
            train::adapt(&mut w, 2, false, 2);
        }
        train::adapt(&mut w, 0, true, 2);
        check(w == vec![3, 4, 14, 4], format!("adaptive update: five failures on option 3 at rate 2, one success on option 1: weights {w:?}"));
    }

    // ---- on the engine: the weights shift after scripted failures; deterministic
    let fail_drill = train::parse("name: check fail\ncpu: none\nadapt: tech\noptions: in_place roll_in roll_away miss\nweights: 4 4 4 4\ntrigger: start\nsuccess: hit 2\ntimeout: 120\nloop: after 4\n").unwrap();
    let pass_drill = train::parse("name: check pass\ncpu: none\nadapt: tech\noptions: in_place roll_in roll_away miss\nweights: 4 4 4 4\ntrigger: start\nsuccess: safe 2\ntimeout: 120\nloop: after 4\n").unwrap();
    let fi = lab.drill_add(fail_drill);
    let pi = lab.drill_add(pass_drill);
    lab.apply(&c(OP_GO_LIVE, 0, 0, 0, 0, 0));
    lab.apply(&c(OP_CPU, 1, CPU_NONE as u16, 0, 0, 0));
    lab.apply(&c(OP_SAVE, 0, 6, 0, 0, 0));
    let mut runs = Vec::new();
    for _ in 0..2 {
        let h = run(lab, 160, &[c(OP_LOAD, 0, 6, 0, 0, 0), c(OP_RNG_SEED, 0, 0, 99, 0, 0), c(OP_ADAPT, 0, ADAPT_WEAK, 3, 0, 0), c(OP_DRILL, 1, fi as u16, 0, 0, 0)]);
        runs.push((h, lab.drill_state()));
        lab.apply(&c(OP_DRILL, 1, DRILL_STOP, 0, 0, 0));
    }
    let (w, n, ok) = runs[0].1.clone().unwrap_or_default();
    check(n >= 5 && ok == 0 && w.iter().map(|&x| x as u32).sum::<u32>() > 16 && w.iter().any(|&x| x > 4), format!("weights shift after scripted failures: {n} attempts, {ok} successes, weights 4 4 4 4 -> {w:?}"));
    check(diff(&runs[0].0, &runs[1].0).is_none() && runs[0].1 == runs[1].1, format!("drill runner deterministic: two runs from slot 7 with seed 99 equal (first difference {:?})", diff(&runs[0].0, &runs[1].0)));
    run(lab, 160, &[c(OP_LOAD, 0, 6, 0, 0, 0), c(OP_DRILL, 1, pi as u16, 0, 0, 0)]);
    let (w2, n2, ok2) = lab.drill_state().unwrap_or_default();
    check(n2 >= 5 && ok2 == n2 && w2.iter().all(|&x| x < 4), format!("successes take weight away: {n2} attempts, {ok2} successes, weights {w2:?}"));
    lab.apply(&c(OP_DRILL, 1, DRILL_STOP, 0, 0, 0));
    lab.apply(&c(OP_ADAPT, 0, ADAPT_OFF, 0, 0, 0));
    {
        let s = sh.borrow();
        eprintln!("lab-check: info drill status after stop: active {} error {}", s.drill[0], s.drill[15]);
    }
    // a starter drill on the engine (out of shield: the dummy shields the scripted P1): attempts loop
    let h = run(lab, 600, &[c(OP_LOAD, 0, 3, 0, 0, 0), c(OP_DRILL, 1, 4, 0, 0, 0)]);
    let st = lab.drill_state();
    eprintln!("lab-check: info `Out of shield` on the scripted P1 for 600 frames: (weights, attempts, successes) {st:?}, last hash {:08X}", h.last().copied().unwrap_or(0));
    lab.apply(&c(OP_DRILL, 1, DRILL_STOP, 0, 0, 0));
    lab.apply(&c(OP_CPU, 1, CPU_NONE as u16, 0, 0, 0));

    // ---- mirror and jitter on load
    let plain = run(lab, 120, &[c(OP_RNG_SEED, 0, 0, 5, 0, 0), c(OP_LOAD, 0, 6, 0, 0, 0)]);
    lab.apply(&c(OP_LOAD_VARY, 1, VARY_MIRROR, 40, 20, 0));
    lab.apply(&c(OP_LOAD_VARY, 1, 0, 0, 0, 0));
    let off = run(lab, 120, &[c(OP_RNG_SEED, 0, 0, 5, 0, 0), c(OP_LOAD, 0, 6, 0, 0, 0)]);
    check(diff(&plain, &off).is_none(), format!("mirror / jitter disabled: the load is hash-equal to a plain load (first difference {:?})", diff(&plain, &off)));
    lab.apply(&c(OP_LOAD_VARY, 1, VARY_MIRROR, 40, 20, 0));
    let on1 = run(lab, 120, &[c(OP_RNG_SEED, 0, 0, 5, 0, 0), c(OP_LOAD, 0, 6, 0, 0, 0)]);
    let on2 = run(lab, 120, &[c(OP_RNG_SEED, 0, 0, 5, 0, 0), c(OP_LOAD, 0, 6, 0, 0, 0)]);
    check(diff(&on1, &plain).is_some() && diff(&on1, &on2).is_none(), format!("mirror + jitter on: differs from the plain load (from frame {:?}), equal twice from one seed: {}", diff(&on1, &plain), diff(&on1, &on2).is_none()));
    lab.apply(&c(OP_LOAD_VARY, 1, 0, 0, 0, 0));

    // ---- the session log (presentation): attempts were written
    let p = train::lab_dir().map(|d| d.join("drill-log.tsv"));
    let rows = p.as_ref().and_then(|p| std::fs::read_to_string(p).ok()).map_or(0, |s| s.lines().filter(|l| l.contains("check fail")).count());
    check(rows >= 5, format!("session log: {rows} rows of the check drill in {:?}", p));
}
