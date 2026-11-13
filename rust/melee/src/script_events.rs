//! `melee script-events --fighter NAME --motion MSID --out FILE [--frames N] [--iso ISO] [--mods DIR [--enable a,b]]`:
//! headless per-frame log of what a move's action script did, to check script overrides (notes/script-overrides.md).
//!
//! Boots a VS match on Final Destination with idle pads (as pose-dump does), waits for P1 to stand in Wait, enters motion
//! state `MSID` (`Fighter_ChangeMotionState`, frame 0, rate 1, no blend) and ticks until the fighter leaves that state or
//! N frames pass. Each line of FILE is one frame (1 = read right after the state was entered, before a tick; measured: a
//! script hitbox at script frame F shows on line F + 1, the same offset with and without overrides):
//! `{"frame", "motion", "anim_id", "anim_frame", "hitboxes": [active slots], "iasa": 0|1}`; the trailing `events` lines
//! summarise the frames where hitboxes came on / went off and where IASA was first set.
//!
//! Its own process, its own engine: nothing here runs in normal play.
#![allow(static_mut_refs)]

use melee_types::*;
use std::vec::Vec;
use std::process::ExitCode;

pub fn script_events(args: &[String]) -> ExitCode {
    let arg = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let (Some(fighter), Some(motion), Some(out)) = (arg("--fighter"), arg("--motion").and_then(|s| s.parse::<i32>().ok()), arg("--out")) else {
        eprintln!("usage: melee script-events --fighter NAME --motion MSID --out FILE [--frames N] [--iso ISO] [--mods DIR [--enable a,b]]");
        return ExitCode::from(2);
    };
    let frames: usize = arg("--frames").and_then(|s| s.parse().ok()).unwrap_or(120);
    let Some(iso) = arg("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok()) else {
        eprintln!("script-events: no --iso / GW_ISO_VANILLA");
        return ExitCode::from(2);
    };
    if melee_engine::scenario::ckind_by_name(&fighter).is_none() {
        eprintln!("script-events: unknown fighter {fighter:?}");
        return ExitCode::from(2);
    }
    if arg("--mods").is_some() {
        if let Err(er) = crate::set_engine_mods(args) {
            eprintln!("script-events: mods: {er}");
            return ExitCode::FAILURE;
        }
    }
    let other = if fighter == "fox" { "falco" } else { "fox" };
    let scene = format!("mode=vs;at=match;p1={fighter}/human;p2={other}/human;p3=none;p4=none;stage=fd;match=time;minutes=0;items=off");
    // the disc error text could name the image: not printed
    let Ok(disc) = crate::engine_disc(&iso) else {
        eprintln!("script-events: cannot open the disc image");
        return ExitCode::FAILURE;
    };
    let scn = melee_engine::scenario::Scenario::parse(&scene).expect("scene");
    let mut e = match melee_engine::Engine::boot_vs(disc, 64 << 20, melee_platform::PadSystem::headless(), scn) {
        Ok(e) => e,
        Err(er) => {
            eprintln!("script-events: engine boot: {er}");
            return ExitCode::FAILURE;
        }
    };
    let fighter_now = || unsafe { melee_modes::tu::script_game::script_fighter(0) };
    let mut ready = false;
    for i in 0..900 {
        e.tick();
        let fp = fighter_now();
        if i > 30 && !fp.is_null() && unsafe { (*fp).motion_id } == ftCo_MS_Wait as _ && unsafe { (*fp).ground_or_air } == GA_Ground {
            ready = true;
            break;
        }
    }
    if !ready {
        eprintln!("script-events: P1 never reached Wait");
        return ExitCode::FAILURE;
    }
    let fp = fighter_now();
    let line = |n: usize| -> (String, Vec<usize>, bool) {
        unsafe {
            let f = &*fp;
            let hb: Vec<usize> = (0..4).filter(|&i| f.x914[i].state as u32 != 0).collect();
            let iasa = f.allow_interrupt() != 0;
            let s = format!(
                "{{\"frame\":{n},\"motion\":{},\"anim_id\":{},\"anim_frame\":{:?},\"hitboxes\":{:?},\"iasa\":{}}}",
                f.motion_id as i32, f.anim_id, f.cur_anim_frame, hb, iasa as u8
            );
            (s, hb, iasa)
        }
    };
    unsafe { melee_fighter::tu::fighter::Fighter_ChangeMotionState((*fp).gobj as _, motion, 0, 0.0, 1.0, -1.0, core::ptr::null_mut()) };
    let mut out_lines = Vec::new();
    let (mut events, mut prev_hb, mut had_iasa) = (Vec::new(), Vec::<usize>::new(), false);
    let mut n = 1;
    loop {
        let (s, hb, iasa) = line(n);
        out_lines.push(s);
        if hb != prev_hb {
            events.push(format!("{{\"event\":\"hitboxes\",\"frame\":{n},\"active\":{hb:?}}}"));
            prev_hb = hb;
        }
        if iasa && !had_iasa {
            events.push(format!("{{\"event\":\"iasa\",\"frame\":{n}}}"));
            had_iasa = true;
        }
        if n >= frames || unsafe { (*fp).motion_id } as i32 != motion {
            break;
        }
        e.tick();
        n += 1;
    }
    let left = unsafe { (*fp).motion_id } as i32;
    events.push(format!("{{\"event\":\"end\",\"frame\":{n},\"motion\":{left}}}"));
    for ev in &events {
        println!("{ev}");
    }
    out_lines.extend(events);
    if let Err(er) = std::fs::write(&out, out_lines.join("\n") + "\n") {
        eprintln!("script-events: {out}: {er}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
