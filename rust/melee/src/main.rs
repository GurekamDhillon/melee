//! `melee`: the native executable.
//!
//! There is no playable game yet: no file loader, no HSD object system, no fighter/stage tick.
//! What this binary has is the headless frame driver the real sim will plug into, and the
//! per-frame neutral-hash CSV writer (format NHV1, docs/rust-port/neutral-hash.md) that
//! `tools/rust-port/compare_golden.py` checks against the C build's goldens.
//!
//! ```text
//! melee selftest
//! melee hashlog --words GOLDEN.csv --out CANDIDATE.csv [--frames N]
//! ```
//!
//! `hashlog --words` is a PLUMBING check, not a simulation: the per-fighter and item words
//! (s0..s11, items, itemsum) and the seed are read back from an existing CSV, and only the frame
//! combine (`nh_game`) and the CSV writer are Rust. When a real `Sim` exists it replaces
//! `WordReplay` and the same writer/compare path applies unchanged.
use melee_core::hash::{StateHash, StateHasher};
use melee_types::neutral_hash::nh_game;
use std::io::Write;
use std::process::ExitCode;

/// What the frame driver needs from a simulation, per frame.
trait Sim {
    /// Advance one frame. Returns false when the run is over.
    fn step(&mut self) -> bool;
    /// Neutral-hash inputs for the frame just simulated.
    fn frame_words(&self) -> FrameWords;
}

/// The values `RB_NeutralHash` combines for one frame.
#[derive(Clone, Copy, Default)]
struct FrameWords {
    frame: u32,
    seed: u32,
    items: u32,
    itemsum: u32,
    slots: [Option<u32>; 12],
}

/// Feeds a recorded CSV's words back through the driver (see module docs: not a simulation).
struct WordReplay {
    rows: std::vec::Vec<FrameWords>,
    pos: usize,
}

impl WordReplay {
    fn load(path: &str) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let mut lines = text.lines();
        let header = lines.next().unwrap_or("");
        if !header.starts_with("frame,hash,seed,items,itemsum,s0") {
            return Err(format!("{path}: not an NHV1 CSV"));
        }
        let hx = |s: &str| u32::from_str_radix(s.trim(), 16).map_err(|e| format!("{path}: {s:?}: {e}"));
        let mut rows = std::vec::Vec::new();
        for l in lines.filter(|l| !l.trim().is_empty()) {
            let c: std::vec::Vec<&str> = l.trim().split(',').collect();
            if c.len() != 17 {
                return Err(format!("{path}: bad row {l:?}"));
            }
            let mut w = FrameWords {
                frame: c[0].parse().map_err(|e| format!("{path}: {e}"))?,
                seed: hx(c[2])?,
                items: c[3].parse().map_err(|e| format!("{path}: {e}"))?,
                itemsum: hx(c[4])?,
                slots: [None; 12],
            };
            for i in 0..12 {
                let v = hx(c[5 + i])?;
                // The C hashlog writes 0 for an absent slot (neutral-hash.md).
                w.slots[i] = (v != 0).then_some(v);
            }
            rows.push(w);
        }
        Ok(Self { rows, pos: 0 })
    }
}

impl Sim for WordReplay {
    fn step(&mut self) -> bool {
        self.pos += 1;
        self.pos <= self.rows.len()
    }
    fn frame_words(&self) -> FrameWords {
        self.rows[self.pos - 1]
    }
}

/// The headless loop: step, hash, log. Returns frames written.
fn run(sim: &mut dyn Sim, out: &mut dyn Write, max_frames: Option<u32>) -> std::io::Result<u32> {
    writeln!(out, "frame,hash,seed,items,itemsum,s0,s1,s2,s3,s4,s5,s6,s7,s8,s9,s10,s11")?;
    let mut n = 0;
    while max_frames.map_or(true, |m| n < m) && sim.step() {
        let w = sim.frame_words();
        let h = nh_game(w.seed, &w.slots, w.items, w.itemsum);
        write!(out, "{},{:08X},{:08X},{},{:08X}", w.frame, h, w.seed, w.items, w.itemsum)?;
        for s in w.slots {
            write!(out, ",{:08X}", s.unwrap_or(0))?;
        }
        writeln!(out)?;
        n += 1;
    }
    Ok(n)
}

fn selftest() {
    // Touch each crate so the link is real.
    let _ = (melee_types::melee_core::VERSION, melee_runtime::melee_core::VERSION,
             melee_world::melee_core::VERSION, melee_fighter::melee_core::VERSION,
             melee_chara::melee_core::VERSION, melee_items::melee_core::VERSION,
             melee_modes::melee_core::VERSION);
    let mut h = StateHasher::new();
    1.0f32.state_hash(&mut h);
    println!("melee (rust port) core {} hash-selftest {:016x}", melee_core::VERSION, h.finish());
    println!("no game simulation is ported yet; see RUST_PORT.md");
}

fn main() -> ExitCode {
    let args: std::vec::Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None | Some("selftest") => {
            selftest();
            ExitCode::SUCCESS
        }
        Some("hashlog") => {
            let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
            let (Some(words), Some(out)) = (get("--words"), get("--out")) else {
                eprintln!("usage: melee hashlog --words GOLDEN.csv --out CANDIDATE.csv [--frames N]");
                return ExitCode::from(2);
            };
            let frames = get("--frames").and_then(|s| s.parse().ok());
            let mut sim = match WordReplay::load(&words) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("melee: {e}");
                    return ExitCode::from(2);
                }
            };
            let res = std::fs::File::create(&out).and_then(|f| {
                let mut w = std::io::BufWriter::new(f);
                let n = run(&mut sim, &mut w, frames)?;
                w.flush()?;
                Ok(n)
            });
            match res {
                Ok(n) => {
                    eprintln!("melee: wrote {n} frames to {out} (word replay, not a simulation)");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("melee: {out}: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some(x) => {
            eprintln!("melee: unknown command {x:?} (selftest | hashlog)");
            ExitCode::from(2)
        }
    }
}
