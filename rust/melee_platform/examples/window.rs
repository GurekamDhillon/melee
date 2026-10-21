//! Smoke test: open the window, run the 60 Hz loop with live input, log pad changes.
//!   cargo run -p melee_platform --example window [-- <seconds>] [--keyboard] [--headless N]
//! `--keyboard` binds port 1 to the preset keyboard map. Exits after <seconds> (default 10).
use melee_platform::aurora::KeyboardBindings;
use melee_platform::pad::*;
use melee_platform::{run_headless, run_windowed, Game, PadSystem, WindowConfig};
use std::time::Instant;
use winit::window::Window;

struct Demo {
    frames: u64,
    limit: u64,
    last: [PADStatus; 4],
    presents: u64,
    t0: Instant,
}

impl Game for Demo {
    fn tick(&mut self, pads: &mut PadSystem) {
        let mut st = [PADStatus::zeroed(); 4];
        pads.read(&mut st);
        for i in 0..4 {
            if !pad_eq(&st[i], &self.last[i]) {
                let s = &st[i];
                println!(
                    "frame {:5} port {} src {:?}: err {} btn {:04x} stick ({},{}) c ({},{}) L {} R {}",
                    self.frames, i + 1, pads.sources[i], s.err, s.button, s.stickX, s.stickY, s.substickX, s.substickY, s.triggerLeft, s.triggerRight
                );
                self.last[i] = *s;
            }
        }
        self.frames += 1;
    }
    fn present(&mut self, _w: &Window, _alpha: f64) {
        self.presents += 1;
    }
    fn should_quit(&self) -> bool {
        self.frames >= self.limit
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut secs = 10u64;
    let mut keyboard = false;
    let mut headless = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--keyboard" => keyboard = true,
            "--headless" => headless = it.next().and_then(|v| v.parse().ok()),
            v => secs = v.parse().unwrap_or(secs),
        }
    }
    let mut demo = Demo { frames: 0, limit: secs * 60, last: [PADStatus::zeroed(); 4], presents: 0, t0: Instant::now() };
    if let Some(n) = headless {
        let mut pads = PadSystem::headless();
        let ran = run_headless(&mut demo, &mut pads, n);
        println!("headless: {ran} ticks in {:?}", demo.t0.elapsed());
        return;
    }
    let mut pads = PadSystem::interactive();
    if keyboard {
        pads.keyboard[0] = KeyboardBindings::preset_default();
    }
    let t0 = Instant::now();
    match run_windowed(demo, pads, WindowConfig::default()) {
        Ok((ticks, dropped)) => {
            let el = t0.elapsed().as_secs_f64();
            println!("windowed: {ticks} ticks ({dropped} dropped) in {el:.3} s = {:.2} Hz", ticks as f64 / el);
        }
        Err(e) => eprintln!("event loop error: {e}"),
    }
}
