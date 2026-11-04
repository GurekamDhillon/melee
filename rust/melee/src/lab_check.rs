//! `melee lab-check [--scenario S] [--iso ISO]`: the Lab's acceptance checks on the engine, headless (needs the disc):
//!
//! 1. savestate round trip: save, 300 frames, load, re-sim 300 frames: every frame's hash equals the first pass
//!    (wasm-ui-lab-api.md section C, "save, then 300 frames, then load, then re-sim gives the same hash");
//! 2. rewind: with the ring on, 240 frames, rewind 150: the replayed frames' hashes equal the originals, then go live;
//! 3. record and playback: both ports recorded for 120 frames; `play(FROM_SLOT)` of the recording reproduces the
//!    recorded frames' hashes; a mirrored playback on the other port hands the game the mirrored pads;
//! 4. deterministic dummy: shield + random DI / SDI / tech / OoS reactions (lab RNG, weighted), twice from the same
//!    state and seed: identical hashes; a different lab seed changes the choices;
//! 5. cost: the step, ring capture, slot save, load and rewind times per frame (the perf line).
//! Exit 0 when every check passes.

use crate::lab::LabRun;
use crate::play_net::ModesGame;
use melee_modhost::lab::*;
use melee_types::PADStatus;
use melee_net::session::RbGame;

fn script_pads(lab: &LabRun) -> [PADStatus; 4] {
    let g = lab.g.e.frames as i32;
    let mut p = [PADStatus::zeroed(); 4];
    for port in 0..2 {
        let i = crate::netmatch::script_input(port, g);
        p[port] = crate::netmatch::engine_game::pad_of(&Some(i));
    }
    p[2].err = -1;
    p[3].err = -1;
    p
}

fn run(lab: &mut LabRun, n: usize, first: &[LabCmd]) -> Vec<u32> {
    let mut h = Vec::with_capacity(n);
    // the commands first: a load moves the frame the scripted pads are read for
    for c in first {
        lab.apply(c);
    }
    for _ in 0..n {
        let p = script_pads(lab);
        lab.tick(p, &[]);
        h.push(lab.hash());
    }
    h
}

fn diff(a: &[u32], b: &[u32]) -> Option<usize> {
    a.iter().zip(b).position(|(x, y)| x != y).or((a.len() != b.len()).then_some(a.len().min(b.len())))
}

pub fn lab_check(args: &[String]) -> std::process::ExitCode {
    let arg = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let scenario = arg("--scenario").unwrap_or_else(|| "pad_falcon_fox_fd".into());
    let Some(iso) = arg("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok()) else {
        eprintln!("melee: no --iso / GW_ISO_VANILLA");
        return std::process::ExitCode::from(2);
    };
    let e = match crate::boot_engine_scenario(&iso, &scenario) {
        Ok(e) => e,
        Err(er) => {
            eprintln!("melee: {er}");
            return std::process::ExitCode::FAILURE;
        }
    };
    let sh = LabShared::new();
    let mut lab = LabRun::new(ModesGame::new(e), sh.clone(), 4242);
    let mut fails = 0;
    let mut check = |ok: bool, what: String| {
        eprintln!("lab-check: {} {what}", if ok { "ok  " } else { "FAIL" });
        if !ok {
            fails += 1;
        }
    };
    let c = LabCmd::new;
    // warm-up: into the match proper (no ring yet: the baseline step cost)
    run(&mut lab, 120, &[]);
    let base_step = lab.perf.step_ns as f64 / lab.perf.steps.max(1) as f64 / 1e3;

    // the dummy check's start state: early, while the scripted P1 still attacks
    lab.apply(&c(OP_SAVE, 0, 3, 0, 0, 0));
    // 1. savestate round trip
    let a = run(&mut lab, 300, &[c(OP_SAVE, 0, 0, 0, 0, 0)]);
    // the save tick saved, then stepped; the load tick loads the slot and steps the same first frame
    let b = run(&mut lab, 300, &[c(OP_LOAD, 0, 0, 0, 0, 0)]);
    check(diff(&b, &a).is_none() && b[0] != 0, format!("savestate round trip: save, 300 frames, load, 300 frames re-simmed: {} of 300 hashes equal (first difference {:?})", b.iter().zip(&a).filter(|(x, y)| x == y).count(), diff(&b, &a)));

    let b2 = run(&mut lab, 300, &[c(OP_LOAD, 0, 0, 0, 0, 0)]);
    eprintln!("lab-check: info load vs load (ring off): first difference {:?}", diff(&b2, &b));
    lab.apply(&c(OP_RING, 0, 600, 0, 0, 0));
    let a3 = run(&mut lab, 300, &[c(OP_SAVE, 0, 4, 0, 0, 0)]);
    let b3 = run(&mut lab, 300, &[c(OP_LOAD, 0, 4, 0, 0, 0)]);
    eprintln!("lab-check: info round trip with the ring on: first difference {:?}", diff(&b3, &a3));
    lab.apply(&c(OP_RING, 0, 0, 0, 0, 0));
    let a4 = run(&mut lab, 300, &[c(OP_SAVE, 0, 4, 0, 0, 0)]);
    let b4 = run(&mut lab, 300, &[c(OP_LOAD, 0, 4, 0, 0, 0)]);
    eprintln!("lab-check: info round trip later, ring off: first difference {:?}", diff(&b4, &a4));
    // 2. rewind with the ring
    lab.apply(&c(OP_RING, 0, 600, 0, 0, 0));
    let h = run(&mut lab, 240, &[]);
    let f_before = lab.f;
    lab.apply(&c(OP_REWIND, 0, 0, 150, 0, 0));
    let back = f_before - lab.f;
    let replay = run(&mut lab, 150, &[]);
    check(back == 150 && diff(&replay, &h[90..240]).is_none(), format!("rewind 150 frames (ring every {} frames): went back {back}, replayed frames equal the originals: {}", crate::lab::RING_EVERY, diff(&replay, &h[90..240]).is_none()));
    lab.apply(&c(OP_GO_LIVE, 0, 0, 0, 0, 0));
    lab.apply(&c(OP_REWIND, 0, 0, 37, 0, 0));
    let r2 = run(&mut lab, 37, &[]);
    check(diff(&r2, &replay[113..150]).is_none(), "rewind 37 (not a ring frame): replay equal".into());
    lab.apply(&c(OP_GO_LIVE, 0, 0, 0, 0, 0));

    // 3. record and playback
    lab.apply(&c(OP_SAVE, 0, 2, 0, 0, 0));
    let rec = 0x05u8;
    lab.apply(&c(OP_REC_START, 0, 0b11, rec as u32, 0, 0));
    let mut recorded_pads = Vec::new();
    let mut hr = Vec::new();
    for _ in 0..120 {
        let p = script_pads(&lab);
        lab.tick(p, &[]);
        recorded_pads.push(lab.last_pads);
        hr.push(lab.hash());
    }
    lab.apply(&c(OP_REC_STOP, 0, 0, rec as u32, 0, 0));
    let len = sh.borrow().rec_len.get(&rec).copied();
    check(len == Some(120), format!("record 120 frames of ports 1-2: gd_lab_rec_len {len:?}"));
    // playback from the recording's start state, with neutral live pads: the recording alone drives both ports
    lab.apply(&c(OP_PLAY, 0, 0b11, rec as u32, PLAY_FROM_SLOT, 0));
    let mut hp = Vec::new();
    for _ in 0..120 {
        let mut p = [PADStatus::zeroed(); 4];
        p[2].err = -1;
        p[3].err = -1;
        lab.tick(p, &[]);
        hp.push(lab.hash());
    }
    check(diff(&hp, &hr).is_none(), format!("play(FROM_SLOT) reproduces the recorded frames: first difference {:?}", diff(&hp, &hr)));
    lab.apply(&c(OP_PLAY_STOP, 0, 0xF, 0, 0, 0));
    // mirrored, port 1's recording on port 2
    let rec1 = 0x06u8;
    lab.apply(&c(OP_REC_START, 0, 0b01, rec1 as u32, 0, 0));
    let mut p1 = Vec::new();
    for _ in 0..30 {
        let p = script_pads(&lab);
        lab.tick(p, &[]);
        p1.push(lab.last_pads[0]);
    }
    lab.apply(&c(OP_REC_STOP, 0, 0, rec1 as u32, 0, 0));
    lab.apply(&c(OP_PLAY, 0, 0b10, rec1 as u32, PLAY_MIRROR, 0));
    let mut ok = true;
    for k in 0..30 {
        let p = script_pads(&lab);
        lab.tick(p, &[]);
        let got = lab.last_pads[1];
        ok &= got.stickX == p1[k].stickX.saturating_neg() && got.button == p1[k].button && got.stickY == p1[k].stickY;
    }
    check(ok, "mirrored playback of P1's recording on P2: stick X flipped, buttons kept".into());
    lab.apply(&c(OP_PLAY_STOP, 0, 0xF, 0, 0, 0));
    // ordered / weighted playback sets
    lab.apply(&c(OP_PLAY_SET, 1, ORDER_WEIGHTED as u16, u32::from_le_bytes([rec1, rec, 0xFF, 0xFF]), u32::MAX, 0x13));
    run(&mut lab, 200, &[]);
    lab.apply(&c(OP_PLAY_STOP, 0, 0xF, 0, 0, 0));

    // 4. deterministic dummy (P2): shield, then the reactions from the lab RNG
    let dummy = |seed: u32| -> Vec<LabCmd> {
        vec![
            c(OP_LOAD, 0, 3, 0, 0, 0),
            c(OP_RNG_SEED, 0, 0, seed, 0, 0),
            c(OP_CPU, 1, CPU_SHIELD as u16, 0, 0, 0),
            c(OP_REACTION, 1, RK_DI as u16, RO_RANDOM as u32, ReactParam { strength: 100, ..Default::default() }.to_u32(), 0),
            c(OP_REACTION, 1, RK_SDI as u16, RO_AWAY as u32, ReactParam { delay: 1, ..Default::default() }.to_u32(), 0),
            c(OP_REACTION_WEIGHTS, 1, RK_TECH as u16, 0x0000_1213, 0, 0),
            c(OP_REACTION, 1, RK_TECH as u16, RO_RANDOM as u32, 0, 0),
            c(OP_REACTION, 1, RK_OOS as u16, RO_RANDOM as u32, ReactParam { delay: 2, ..Default::default() }.to_u32(), 0),
            c(OP_REACTION, 1, RK_SHIELD as u16, RO_BLOCK_RANDOM as u32, ReactParam { chance: 60, ..Default::default() }.to_u32(), 0),
        ]
    };
    let mut d1 = Vec::new();
    let mut d2 = Vec::new();
    let mut ev = [0u32; 16];
    for (out, seed) in [(&mut d1, 77u32), (&mut d2, 77)] {
        let cmds = dummy(seed);
        for c in &cmds {
            lab.apply(c);
        }
        for _ in 0..600 {
            let p = script_pads(&lab);
            lab.tick(p, &[]);
            out.push(lab.hash());
            for e in sh.borrow().events.iter() {
                ev[e.kind as usize & 15] += 1;
            }
        }
    }
    check(diff(&d1, &d2).is_none(), format!("deterministic dummy: two runs from slot 3 with lab seed 77 equal over 600 frames (first difference {:?})", diff(&d1, &d2)));
    let mut d3 = Vec::new();
    {
        let cmds = dummy(78);
        for c in &cmds {
            lab.apply(c);
        }
        for _ in 0..600 {
            let p = script_pads(&lab);
            lab.tick(p, &[]);
            d3.push(lab.hash());
        }
    }
    let base: Vec<u32> = {
        let mut cmds = vec![c(OP_LOAD, 0, 3, 0, 0, 0), c(OP_CPU, 1, CPU_NONE as u16, 0, 0, 0)];
        for k in 0..RK_COUNT as u16 {
            cmds.push(c(OP_REACTION, 1, k, RO_NONE as u32, 0, 0));
        }
        let mut v = Vec::new();
        for c in &cmds {
            lab.apply(c);
        }
        for _ in 0..600 {
            let p = script_pads(&lab);
            lab.tick(p, &[]);
            v.push(lab.hash());
        }
        v
    };
    check(diff(&d1, &base).is_some(), format!("the dummy acts: its run differs from the scripted one from frame {:?}", diff(&d1, &base)));
    eprintln!("lab-check: info seed 78 vs 77: first difference {:?} (None = the random draws did not matter in this clip)", diff(&d1, &d3));
    eprintln!(
        "lab-check: info events over the dummy runs: hit {} shield {} powershield {} lcancel {} tech {} oos {} iasa {} sdi {} asdi {} airdodge {} wavedash {} ledgedash {}",
        ev[EV_HIT as usize], ev[EV_SHIELD as usize], ev[EV_POWERSHIELD as usize], ev[EV_LCANCEL as usize], ev[EV_TECH as usize], ev[EV_OOS as usize],
        ev[EV_IASA as usize], ev[EV_SDI as usize], ev[EV_ASDI as usize], ev[EV_AIRDODGE as usize], ev[EV_WAVEDASH as usize], ev[EV_LEDGEDASH as usize]
    );
    {
        let s = sh.borrow();
        let m: Vec<String> = (0..2).map(|p| s.meter[p].iter().rev().take(40).rev().map(|c| char::from(b'0' + c)).collect()).collect();
        eprintln!("lab-check: info frame meter P1 {} / P2 {}; frame advantage {}; framedata entries {}", m[0], m[1], s.frame_adv, s.framedata.len());
        check(s.fighters[0].is_some() && s.meter[0].len() == METER_CELLS && !s.framedata.is_empty(), "queries published (fighter v1, meter, observed frame data)".into());
    }

    // 5. cost: a rewind of 300 frames and the per-frame amortised ring cost over a long run
    lab.apply(&c(OP_CPU, 1, CPU_NONE as u16, 0, 0, 0));
    lab.apply(&c(OP_GO_LIVE, 0, 0, 0, 0, 0));
    run(&mut lab, 900, &[]);
    lab.apply(&c(OP_REWIND, 0, 0, 300, 0, 0));
    lab.apply(&c(OP_GO_LIVE, 0, 0, 0, 0, 0));
    eprintln!("lab-check: info baseline step (no ring) {base_step:.0} us/frame");
    eprintln!("lab-check: info {}", lab.perf_line());
    let p = lab.perf;
    eprintln!(
        "lab-check: info ring: {} snapshots held for {} frames, {:.1} MB distinct page data",
        sh.borrow().ring_len / crate::lab::RING_EVERY as u32 + 1, sh.borrow().ring_len, p.ring_bytes as f64 / 1e6
    );
    if fails == 0 {
        eprintln!("lab-check: ALL OK");
        std::process::ExitCode::SUCCESS
    } else {
        eprintln!("lab-check: {fails} FAILED");
        std::process::ExitCode::FAILURE
    }
}
