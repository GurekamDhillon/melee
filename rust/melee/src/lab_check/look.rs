//! lab-check part 6: the lookahead (lab-theorycraft.md 3b / 4c). Hash-equal after every lookahead, no observer side
//! effects, the no-DI and on-shield results against playing the frames out, the RNG sweep, live-mode slicing cost.

use super::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

fn neutral_pads() -> [PADStatus; 4] {
    let mut p = [PADStatus::zeroed(); 4];
    p[2].err = -1;
    p[3].err = -1;
    p
}

/// Speculative-tick probe (engine FRAME_POST_HOOK): counts in a system-heap cell (a static would be rewound by the load).
static PROBE: std::sync::OnceLock<&'static [AtomicU64; 2]> = std::sync::OnceLock::new();
fn probe_hook() {
    if let Some(p) = PROBE.get() {
        if melee_engine::engine::speculative() {
            p[0].fetch_add(1, Relaxed);
            if melee_audio::sim::is_installed() {
                p[1].fetch_add(1, Relaxed);
            }
        }
    }
}

/// A fixed exchange from slot 3: P1 and P2 (the dummy `cpu`, at `pct` %) placed mid-stage side by side, both land, P1 forward-smashes
/// (C-stick); ticks until the event `kind` on P2. True when it came.
fn exchange(lab: &mut LabRun, sh: &Rc<RefCell<LabShared>>, cpu: i32, pct: f32, kind: u8) -> bool {
    let c = LabCmd::new;
    lab.apply(&c(OP_LOAD, 0, 3, 0, 0, 0));
    lab.apply(&c(OP_CPU, 1, cpu as u16, 0, 0, 0));
    for _ in 0..20 {
        lab.tick(neutral_pads(), &[]);
    }
    let f = sh.borrow().fighters[0].unwrap();
    let rd = |i: usize| f32::from_le_bytes(f[i * 4..i * 4 + 4].try_into().unwrap());
    // mid-stage (FD's floor is y 0), P2 on the side P1 faces
    let face = rd(19);
    let x = -10.0 * face;
    lab.apply(&c(OP_POS, 0, 0, x.to_bits(), 4.0f32.to_bits(), 0));
    lab.apply(&c(OP_POS, 1, 0, (x + 14.0 * face).to_bits(), 4.0f32.to_bits(), 0));
    lab.apply(&c(OP_PERCENT, 1, 0, pct.to_bits(), 0, 0));
    for _ in 0..40 {
        lab.tick(neutral_pads(), &[]);
    }
    let mut p = neutral_pads();
    p[0].substickX = if face > 0.0 { 127 } else { -127 };
    if std::env::var("LOOK_DEBUG").is_ok() {
        let s = sh.borrow();
        for q in 0..2 {
            let f = s.fighters[q].unwrap();
            let rd = |i: usize| f32::from_le_bytes(f[i * 4..i * 4 + 4].try_into().unwrap());
            let ri = |i: usize| i32::from_le_bytes(f[i * 4..i * 4 + 4].try_into().unwrap());
            eprintln!("lab-check: debug exchange P{} action {} x {} y {} air {} facing {}", q + 1, ri(3), rd(14), rd(15), ri(13), rd(19));
        }
    }
    for i in 0..90 {
        lab.tick(if i < 2 { p } else { neutral_pads() }, &[]);
        if std::env::var("LOOK_DEBUG").is_ok() && i < 40 {
            let s = sh.borrow();
            let a = |q: usize| i32::from_le_bytes(s.fighters[q].unwrap()[12..16].try_into().unwrap());
            eprintln!("lab-check: debug  +{i} P1 {} P2 {} events {:?}", a(0), a(1), s.events.iter().map(|e| (e.kind, e.port)).collect::<Vec<_>>());
        }
        if sh.borrow().events.iter().any(|e| e.kind == kind && e.port == 1) {
            return true;
        }
    }
    false
}

/// Play the anchored exchange out for real with neutral live pads (P2's dummy may still shield): the first actionable
/// (or KO) frame of P1 and P2 after the anchor.
fn play_out(lab: &mut LabRun) -> [Option<i32>; 2] {
    let f0 = lab.f;
    let mut at = [lab.actionable_now(0).then_some(0), lab.actionable_now(1).then_some(0)];
    for _ in 0..LOOK_CAP {
        if at.iter().all(|a| a.is_some()) {
            break;
        }
        lab.tick(neutral_pads(), &[]);
        for p in 0..2 {
            if at[p].is_none() && lab.actionable_now(p) {
                at[p] = Some(lab.f - f0);
            }
        }
    }
    at
}

/// Before the lab takes the engine (the snapshot base): a snapshot load restores every static, so the probe's cell
/// address and the frame hook must be set before the base is captured, or the first load would clear them.
pub(super) fn init_probe() {
    let _ = PROBE.set(Box::leak(Box::new([AtomicU64::new(0), AtomicU64::new(0)])));
    unsafe { melee_engine::engine::FRAME_POST_HOOK = Some(probe_hook) };
}

pub(super) fn lookahead_checks(lab: &mut LabRun, sh: &Rc<RefCell<LabShared>>, check: &mut dyn FnMut(bool, String)) {
    let c = LabCmd::new;
    lab.look_verify = true;
    sh.borrow_mut().look_cfg = LOOK_AUTO;
    lab.apply(&c(OP_RING, 0, 600, 0, 0, 0));
    lab.apply(&c(OP_GO_LIVE, 0, 0, 0, 0, 0));

    // (a) P1's forward smash hits P2 at 110 %: the auto job is pending (live mode off); run it as a frozen tick would
    let hit = exchange(lab, sh, CPU_NONE, 130.0, EV_HIT);
    check(hit && lab.look_pending(), format!("lookahead: a hit on P2 queues the auto hit job (hit seen {hit})"));
    if !hit {
        return;
    }
    let (h0, fh0, d0) = (lab.g.state_hash(), lab.frame_hash(), lab.observer_digest());
    // an (unbooted) sound installed: it must be out of the static for every speculative tick, and back after
    melee_audio::sim::install(melee_audio::Sound::new());
    let t = std::time::Instant::now();
    lab.look_run(None);
    let ms = t.elapsed().as_secs_f64() * 1e3;
    let sound_back = melee_audio::sim::uninstall().is_some();
    let d1 = lab.observer_digest();
    let (anchor, runs) = lab.look_result().unwrap_or((0, Vec::new()));
    let ticks = sh.borrow().look[10] as u64;
    let pr = PROBE.get().unwrap();
    let (spec_frames, spec_sound) = (pr[0].load(Relaxed), pr[1].load(Relaxed));
    check(lab.g.state_hash() == h0 && lab.frame_hash() == fh0 && lab.look_perf.verify_bad == 0, format!("lookahead hit job: state hash equal after ({h0:08x})"));
    check(d1 == d0, "lookahead fires no observer side effects (events, meter, frame data, frame_adv, ring, log, recordings, playback, dummy, lab RNG, perf, fighters)".into());
    check(
        spec_frames == ticks && spec_sound == 0 && lab.look_perf.sound_leaks == 0 && sound_back,
        format!("speculative ticks: {spec_frames} engine frames flagged (job ticks {ticks}), {spec_sound} with the sound installed, sound back after: {sound_back}"),
    );
    check(runs.len() == LOOK_RUNS && runs.iter().all(|r| r[1] != LO_NOT_RUN), format!("DI fan: {} runs done", runs.len()));
    eprintln!(
        "lab-check: info lookahead hit job at frame {anchor}: {ticks} ticks in {ms:.2} ms ({:.2} ms per run, {:.0} us per tick incl. save/load)",
        ms / runs.len().max(1) as f64,
        ms * 1e3 / ticks.max(1) as f64
    );
    let names = ["", "down-left", "down", "down-right", "left", "none", "right", "up-left", "up", "up-right"];
    for r in &runs {
        eprintln!(
            "lab-check: info   DI {:<10} P2 {} at +{}, P1 acts +{} (P1 advantage {:+})",
            names[r[0] as usize], ["-", "acts", "KO", "cap"][r[1] as usize], r[2], r[3], r[2] - r[3]
        );
    }
    // (b) the no-DI run against playing the frames out
    let nodi = runs.iter().find(|r| r[0] == 5).copied().unwrap_or([0; 4]);
    let real = play_out(lab);
    check(
        real == [Some(nodi[3]), Some(nodi[2])],
        format!("lookahead on hit equals playing it out: lookahead P1 +{} P2 +{} (advantage {:+}); played out {real:?}; observed frame_adv {:+}", nodi[3], nodi[2], nodi[2] - nodi[3], sh.borrow().frame_adv),
    );

    // (c) on shield: P2's dummy holds shield (the same pad as the lookahead's shield policy)
    if exchange(lab, sh, CPU_SHIELD, 0.0, EV_SHIELD) && lab.look_pending() {
        let h = lab.g.state_hash();
        lab.look_run(None);
        let r = lab.look_result().map(|r| r.1[0]).unwrap_or([0; 4]);
        let ok_hash = lab.g.state_hash() == h;
        let real = play_out(lab);
        check(
            ok_hash && real == [Some(r[3]), Some(r[2])],
            format!("lookahead on shield equals playing it out: lookahead P1 +{} P2 +{} (P1 advantage {:+}); played out {real:?}; hash equal {ok_hash}", r[3], r[2], r[2] - r[3]),
        );
    } else {
        eprintln!("lab-check: info no shield hit in this clip: on-shield comparison skipped");
    }
    lab.apply(&c(OP_CPU, 1, CPU_NONE as u16, 0, 0, 0));

    // (d) RNG sweep (the anchor's own seed, then 8 seeds) on the first hit
    if exchange(lab, sh, CPU_NONE, 150.0, EV_HIT) {
        let h = lab.g.state_hash();
        lab.look_request(LOOK_RNG, 1);
        lab.look_run(None);
        let r = lab.look_result().map(|r| r.1).unwrap_or_default();
        let v: Vec<String> = r.iter().map(|x| format!("{}{}", ["-", "", "KO", "cap"][x[1] as usize], x[2])).collect();
        eprintln!("lab-check: info RNG sweep (anchor seed, then 8 seeds): P2 acts at +{}", v.join(" +"));
        check(r.len() == LOOK_RUNS && lab.g.state_hash() == h, "RNG sweep: 9 seeds run, state hash equal after".into());
    }

    // (e) live mode: sliced under a 4 ms budget per frame; frame times with and without; hashes unchanged
    // the hit, then 300 frames played on (where the live slices run); timed per lab tick
    let run600 = |lab: &mut LabRun| -> (Vec<f64>, Vec<u32>) {
        let (mut t, mut h) = (Vec::new(), Vec::new());
        exchange(lab, sh, CPU_NONE, 110.0, EV_HIT);
        for _ in 0..300 {
            let t0 = std::time::Instant::now();
            lab.tick(neutral_pads(), &[]);
            t.push(t0.elapsed().as_secs_f64() * 1e3);
            h.push(lab.frame_hash());
        }
        (t, h)
    };
    // timing without the hash verify (two whole-state hashes per slice, a test aid); the hashes below still compare
    lab.look_verify = false;
    sh.borrow_mut().look_cfg = 0;
    eprintln!("lab-check: info live-mode timing: the run with the lookahead off");
    let (base, hb) = run600(lab);
    {
        let mut s = sh.borrow_mut();
        s.look_cfg = LOOK_AUTO | LOOK_LIVE;
        s.look_budget_us = 4000;
    }
    let (jobs0, slices0) = (lab.look_perf.jobs, lab.look_perf.slices);
    eprintln!("lab-check: info live-mode timing: the run with the live lookahead");
    let (live, hl) = run600(lab);
    let stat = |v: &[f64]| {
        let mut s = v.to_vec();
        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        (v.iter().sum::<f64>() / v.len() as f64, s[s.len() * 99 / 100], s[s.len() - 1])
    };
    let ((bm, b99, bmax), (lm, l99, lmax)) = (stat(&base), stat(&live));
    eprintln!(
        "lab-check: info frame time (lab tick), lookahead off: mean {bm:.3} ms, p99 {b99:.3}, max {bmax:.3}; live lookahead, 4 ms budget: mean {lm:.3} ms, p99 {l99:.3}, max {lmax:.3}; {} jobs finished in {} slices",
        lab.look_perf.jobs - jobs0, lab.look_perf.slices - slices0
    );
    check(hb == hl, format!("live lookahead: 300 frames after a hit hash-equal to the same frames with it off (first difference {:?})", diff(&hl, &hb)));
    sh.borrow_mut().look_cfg = LOOK_AUTO;
    lab.look_verify = false;
    eprintln!("lab-check: info lookahead totals: {:?}", lab.look_perf);
}
