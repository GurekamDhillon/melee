//! `melee`: the native executable.
//!
//! There is no playable game yet. What this binary has is the headless frame driver, the first
//! native match start (`golden`, vs_match.rs: a few frames of pad_falcon_fox_fd), and the
//! per-frame neutral-hash CSV writer (format NHV1, docs/rust-port/neutral-hash.md) that
//! `tools/rust-port/compare_golden.py` checks against the C build's goldens.
//!
//! ```text
//! melee selftest
//! melee hashlog --words GOLDEN.csv --out CANDIDATE.csv [--frames N]
//! melee golden --iso DISC.iso --out CANDIDATE.csv [--scenario pad_falcon_fox_fd] [--frames N] [--seed-from GOLDEN.csv]
//! ```
//!
//! `hashlog --words` is a PLUMBING check, not a simulation: the per-fighter and item words
//! (s0..s11, items, itemsum) and the seed are read back from an existing CSV, and only the frame
//! combine (`nh_game`) and the CSV writer are Rust. When a real `Sim` exists it replaces
//! `WordReplay` and the same writer/compare path applies unchanged.
//!
//! `golden` IS a simulation (`vs_match::VsMatch`): it loads the stage and common fighter data from
//! the user's disc and steps the ported match code until it reaches code that is not ported, then
//! stops (see vs_match.rs for exactly what runs). Compare its CSV with
//! `compare_golden.py --allow-short`.
use melee_core::hash::{StateHash, StateHasher};
use melee_types::neutral_hash::nh_game;
use std::io::Write;
use std::process::ExitCode;

mod disc;
mod effect;
mod fighter_data;
mod stage;
mod vs_match;

/// What the frame driver needs from a simulation, per frame.
pub(crate) trait Sim {
    /// Advance one frame. Returns false when the run is over.
    fn step(&mut self) -> bool;
    /// Neutral-hash inputs for the frame just simulated.
    fn frame_words(&self) -> FrameWords;
}

/// The values `RB_NeutralHash` combines for one frame.
#[derive(Clone, Copy, Default)]
pub(crate) struct FrameWords {
    pub frame: u32,
    pub seed: u32,
    pub items: u32,
    pub itemsum: u32,
    pub slots: [Option<u32>; 12],
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
        Some("golden") => golden(&args),
        Some(x) => {
            eprintln!("melee: unknown command {x:?} (selftest | hashlog | golden)");
            ExitCode::from(2)
        }
    }
}

/// `melee golden`: the native VS match for a golden scenario (vs_match.rs).
fn golden(args: &[String]) -> ExitCode {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let iso = get("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok());
    let (Some(iso), Some(out)) = (iso, get("--out")) else {
        eprintln!("usage: melee golden --iso DISC.iso --out CANDIDATE.csv [--scenario pad_falcon_fox_fd] [--frames N]");
        return ExitCode::from(2);
    };
    let scenario = get("--scenario").unwrap_or_else(|| "pad_falcon_fox_fd".into());
    // record.sh's scene strings: stage, fighters (P1, P2), MELEE_TEST_SEED. Stocks: VS default 3
    // (the C dump shows stocks=3 in this time match; Player_GetStocks feeds the hash).
    let (stage, fighters): (&str, &[&str]) = match scenario.as_str() {
        "pad_falcon_fox_fd" => ("GrNLa.dat", &["falcon", "fox"]),
        s => {
            eprintln!("melee: scenario {s:?} not wired (pad_falcon_fox_fd)");
            return ExitCode::from(2);
        }
    };
    let frames = get("--frames").and_then(|s| s.parse().ok());
    let res = disc::Disc::open(&iso).and_then(|mut d| vs_match::VsMatch::new(&mut d, stage, fighters, 12345, 3));
    let mut sim = match res {
        Ok(s) => s,
        Err(e) => {
            eprintln!("melee: {e}");
            return ExitCode::FAILURE;
        }
    };
    // Diagnostic only: `--seed-from GOLDEN.csv` writes the golden's seed column instead of the
    // native one (the seed is drawn by the unported particle system), so the remaining columns
    // and the frame combine can be checked on their own. The output then is NOT a pure simulation.
    let seed_from = match get("--seed-from").map(|p| WordReplay::load(&p)).transpose() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("melee: {e}");
            return ExitCode::from(2);
        }
    };
    struct SeedFrom<'a> {
        sim: &'a mut vs_match::VsMatch,
        golden: Option<WordReplay>,
    }
    impl Sim for SeedFrom<'_> {
        fn step(&mut self) -> bool {
            self.sim.step()
        }
        fn frame_words(&self) -> FrameWords {
            let mut w = self.sim.frame_words();
            if let Some(g) = &self.golden {
                if let Some(r) = g.rows.iter().find(|r| r.frame == w.frame) {
                    w.seed = r.seed;
                }
            }
            w
        }
    }
    let injected = seed_from.is_some();
    let res = std::fs::File::create(&out).and_then(|f| {
        let mut w = std::io::BufWriter::new(f);
        let n = run(&mut SeedFrom { sim: &mut sim, golden: seed_from }, &mut w, frames)?;
        w.flush()?;
        Ok(n)
    });
    match res {
        Ok(n) => {
            eprintln!("melee: wrote {n} frames to {out} (native match, {scenario}{})",
                      if injected { "; seed column copied from the golden, diagnostic" } else { "" });
            if let Some(why) = &sim.stopped {
                eprintln!("melee: stopped: {why}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("melee: {out}: {e}");
            ExitCode::FAILURE
        }
    }
}
