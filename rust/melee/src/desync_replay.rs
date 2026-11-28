//! `melee desync-replay DIR [--iso DISC]`: replays a captured desync (`desynccap`) in the engine and prints the engine's
//! Slippi checksum at the detection frame next to the two the match recorded. The replay is a clean re-run of the
//! captured inputs, so the side whose checksum it reproduces is the one whose simulation is the clean one.
//!
//! The inputs come from `DIR/desync.slp` (the Pre-Frame events: raw sticks, buttons, triggers, which are what the
//! simulation read, test perturbations included). The match is booted as the online match it was (Slippi online mode on
//! with the report's `rng_offset`, the online codeset, the scene from the report, `bits.txt`'s seed bytes) and run
//! straight, as `slippi-replay` does, so its checksums are the ones a live run with every input confirmed would produce.
//! `melee slpcheck --slp DIR/desync.slp` also plays the file (it compares against the recorded post-frames).

use crate::netmatch::engine_game::EngineGame;
use crate::netmatch::NetGame as _;
use melee_net::pad::RbInput;
use melee_net::session::{RbGame, FIRST, SLOTS};
use melee_net::slippi_mm::Json;
use std::collections::BTreeMap;
use std::process::ExitCode;

/// Per tick (0 = Slippi frame -123), each port's pad as the Pre-Frame events record it.
pub fn read_pads(d: &[u8]) -> Result<BTreeMap<i32, [Option<RbInput>; 4]>, String> {
    let k = d.windows(8).position(|w| w == b"raw[$U#l").ok_or("no raw block")?;
    let len = u32::from_be_bytes(d.get(k + 8..k + 12).ok_or("short")?.try_into().unwrap()) as usize;
    let raw = d.get(k + 12..k + 12 + len).ok_or("raw block cut short")?;
    if raw.first() != Some(&0x35) {
        return Err("no Event Payloads".into());
    }
    let n = raw[1] as usize;
    let mut sizes = [0usize; 256];
    for e in raw[2..1 + n].chunks_exact(3) {
        sizes[e[0] as usize] = u16::from_be_bytes([e[1], e[2]]) as usize;
    }
    let mut out: BTreeMap<i32, [Option<RbInput>; 4]> = BTreeMap::new();
    let mut at = 1 + n;
    while at < raw.len() {
        let cmd = raw[at];
        let sz = sizes[cmd as usize];
        if sz == 0 && cmd != 0x35 {
            return Err(format!("unknown event {cmd:#x} at {at}"));
        }
        let ev = raw.get(at..at + 1 + sz).ok_or("event cut short")?;
        if cmd == 0x37 && ev.len() > 0x42 && ev[6] == 0 {
            let frame = i32::from_be_bytes(ev[1..5].try_into().unwrap());
            let port = ev[5] as usize & 3;
            let btn = u16::from_be_bytes([ev[0x31], ev[0x32]]);
            let tr = |o: usize| (f32::from_be_bytes(ev[o..o + 4].try_into().unwrap()) * 140.0).round().clamp(0.0, 255.0) as u8;
            let i = RbInput {
                present: 1,
                is_raw: 1,
                confirmed: 1,
                buttons: btn as u32,
                raw: [ev[0x3B] as i8, ev[0x40] as i8, ev[0x41] as i8, ev[0x42] as i8],
                pad_l: tr(0x33),
                pad_r: tr(0x37),
                ..Default::default()
            };
            out.entry(frame + 123).or_default()[port] = Some(i);
        }
        at += 1 + sz;
    }
    Ok(out)
}

pub fn desync_replay(args: &[String]) -> ExitCode {
    let Some(dir) = args.get(1).filter(|a| !a.starts_with("--")) else {
        eprintln!("usage: melee desync-replay DIR [--iso DISC.iso]   (DIR: a desyncs/<timestamp> folder written by a Slippi online match)");
        return ExitCode::from(2);
    };
    let dir = std::path::Path::new(dir);
    let Some(rep) = std::fs::read_to_string(dir.join("report.json")).ok().and_then(|t| Json::parse(&t)) else {
        eprintln!("desync-replay: {}: no readable report.json", dir.display());
        return ExitCode::from(2);
    };
    let Ok(slp) = melee_data::slp::read_replay(dir.join("desync.slp")) else {
        eprintln!("desync-replay: {}: no desync.slp (the capture had no recording)", dir.display());
        return ExitCode::from(2);
    };
    let pads = match read_pads(&slp) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("desync-replay: desync.slp: {e}");
            return ExitCode::from(2);
        }
    };
    let hex = |k: &str| rep.get(k).and_then(|v| v.str()).and_then(|v| u32::from_str_radix(v, 16).ok());
    let (Some(of), Some(lv), Some(rv), Some(m)) = (rep.get("online_frame").and_then(|v| v.num()).map(|n| n as i32), hex("local_checksum"), hex("remote_checksum"), rep.get("match")) else {
        eprintln!("desync-replay: report.json lacks online_frame / local_checksum / remote_checksum / match");
        return ExitCode::from(2);
    };
    let kind = rep.get("kind").and_then(|v| v.str()).unwrap_or("?").to_string();
    let tag = rep.get("tag").and_then(|v| v.str()).unwrap_or("?").to_string();
    let (Some(rng), Some(scene)) = (m.get("rng_offset").and_then(|v| v.num()).map(|n| n as u32), m.get("scene").and_then(|v| v.str())) else {
        eprintln!("desync-replay: report.json: no match.rng_offset / match.scene");
        return ExitCode::from(2);
    };
    let alt = m.get("alt_stage_mode").and_then(|v| v.num()).unwrap_or(0.0) as u8;
    // the seed bytes of the frames where a port rolled a random special
    let mut bits: BTreeMap<i32, [u8; 4]> = BTreeMap::new();
    for l in std::fs::read_to_string(dir.join("bits.txt")).unwrap_or_default().lines() {
        let v: Vec<&str> = l.split_whitespace().collect();
        if v.len() == 5 {
            let b = |i: usize| u8::from_str_radix(v[i], 16).unwrap_or(0);
            bits.insert(v[0].parse().unwrap_or(0), [b(1), b(2), b(3), b(4)]);
        }
    }
    // the ck of online frame `of` is the state after tick `of - 2`
    let ticks = of - 1;
    let last = pads.keys().next_back().copied().unwrap_or(-1);
    if last + 1 < ticks {
        eprintln!("desync-replay: the .slp holds {} ticks, the detection frame needs {ticks}", last + 1);
        return ExitCode::from(2);
    }
    melee_core::slippi_online::enable(rng);
    crate::slippi_np::online_codes(alt);
    let mut game = match EngineGame::open_scene(args, scene, rng) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("desync-replay: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Some(Json::Arr(ps)) = m.get("players") {
        for (i, p) in ps.iter().take(4).enumerate() {
            game.slippi_chars[i] = p.get("character_id").and_then(|v| v.num()).unwrap_or(0.0) as u32;
        }
    }
    for (k, b) in &bits {
        game.slippi_bits.insert(FIRST + k, *b);
    }
    game.slippi_pre_tick();
    game.set_live(None);
    for k in 0..ticks {
        let mut inp = [None; SLOTS];
        if let Some(p) = pads.get(&k) {
            for port in 0..2 {
                inp[port * 2] = p[port];
            }
        }
        RbGame::run_frame(&mut game, &inp, false);
        if game.halted() {
            break;
        }
    }
    // `--ck-out TXT`: every online frame's engine checksum, "<online frame> <slippi frame> <checksum hex>", to find the
    // frame whose state a remote checksum matches (a remote that ran a frame behind or ahead of us)
    if let Some(p) = args.iter().position(|a| a == "--ck-out").and_then(|i| args.get(i + 1)) {
        let t: String = game.slippi_ck.iter().map(|(f, c)| format!("{f} {} {c:08x}\n", crate::desynccap::slippi_frame(*f))).collect();
        if let Err(e) = std::fs::write(p, t) {
            eprintln!("desync-replay: --ck-out {p}: {e}");
        }
    }
    println!("desync-replay: {} ({kind}, {tag} side): online frame {of}, Slippi frame {}", dir.display(), crate::desynccap::slippi_frame(of));
    println!("  recorded local  {lv:08x}");
    println!("  recorded remote {rv:08x}");
    match game.slippi_ck.get(&of) {
        Some(&e) => {
            println!("  replay (engine) {e:08x}");
            let v = match (e == lv, e == rv) {
                (true, false) => "the replay reproduces the LOCAL checksum: this side's state follows from the recorded inputs; the remote side's differs from a clean re-run of them (its simulation, or inputs it saw differently)",
                (false, true) => "the replay reproduces the REMOTE checksum: this side's state differs from a clean re-run of the recorded inputs (a simulation difference on this side)",
                (true, true) => "local and remote are equal (nothing to tell)",
                (false, false) => "the replay matches neither side (the recorded inputs alone do not reproduce either state)",
            };
            println!("  {v}");
            ExitCode::SUCCESS
        }
        None => {
            println!("  replay (engine) none: the run ended before online frame {of}");
            ExitCode::FAILURE
        }
    }
}
