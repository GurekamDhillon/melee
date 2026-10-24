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
//! melee golden --iso DISC.iso --out CANDIDATE.csv [--scenario pad_falcon_fox_fd|pad_falcon_wolf_fd] [--mods DIR [--enable id,id]] [--frames N] [--seed-from GOLDEN.csv] [--audio]
//!              [--from-snapshot FILE.snap [--start N]] [--dump-state N:FILE]
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
mod mex;
mod mods;
mod effect;
mod fighter_data;
mod stage;
mod vs_match;
mod gx_backend;
mod view;
mod snap;
mod fighter_fields;

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
        Some("view-golden") => view::view_golden(&args),
        Some(x) => {
            eprintln!("melee: unknown command {x:?} (selftest | hashlog | golden | view-golden)");
            ExitCode::from(2)
        }
    }
}

/// Boot melee_audio from the disc (smash2.sem + main.ssm, bank sizes from lbaudio_ax.c's table)
/// and set every lbAudioAx hook the ported fighter code calls to it.
fn install_audio(iso: &str) -> Result<(), String> {
    let mut snd = melee_audio::Sound::new();
    snd.boot(Box::new(melee_audio::sound::DiscFs::open(iso)?), melee_audio::lbaudio::boot_bank_sizes())?;
    melee_audio::sim::install(snd);
    use melee_audio::{lbaudio as lb, sim};
    use melee_fighter::hooks as h;
    unsafe {
        h::LBAUDIOAX_800237A8 = Some(sim::lbaudioax_800237a8);
        h::LBAUDIOAX_80023870 = Some(sim::lbaudioax_80023870);
        h::LBAUDIOAX_80024B94 = Some(sim::lbaudioax_80024b94);
        h::LBAUDIOAX_800233EC = Some(sim::lbaudioax_800233ec);
        h::LBAUDIOAX_80023130 = Some(lb::lb_80023130);
        h::LBAUDIOAX_800230C8 = Some(sim::lbaudioax_800230c8);
        h::LBAUDIOAX_80023220 = Some(lb::lb_80023220);
        h::LBAUDIOAX_80024DC4 = Some(sim::lbaudioax_80024dc4);
        h::LBAUDIOAX_80024FDC = Some(sim::lbaudioax_80024fdc);
        h::LBAUDIOAX_80024FF4 = Some(sim::lbaudioax_80024ff4);
        h::LBAUDIOAX_8002500C = Some(sim::lbaudioax_8002500c);
        h::LBAUDIOAX_80025038 = Some(sim::lbaudioax_80025038);
    }
    eprintln!("melee: audio booted (smash2.sem + main.ssm), lbAudioAx hooks set");
    Ok(())
}

/// The match's audio start (lbAudioAx_8002785C + the BGM): the players' and the stage's SSM
/// banks, then the picked BGM.
fn start_match_audio(sim: &vs_match::VsMatch, ckinds: &[i32]) {
    melee_audio::sim::load_match_banks(ckinds, vs_match::STKIND_FD, melee_types::Gr_Kind_Last as i32);
    let r = melee_audio::sim::play_bgm_id(sim.bgm);
    let loaded = melee_audio::sim::with(|s| s.lb.state.iter().enumerate().filter(|(_, &x)| x == 2).map(|(i, _)| melee_audio::lbaudio::SSM_FILES[i]).collect::<Vec<_>>());
    eprintln!("melee: match audio: banks {:?}, bgm {} ({}) {}", loaded.unwrap_or_default(), sim.bgm,
        melee_audio::lbaudio::HPS_FILES.get(sim.bgm as usize).unwrap_or(&"?"), if r == 0 { "started" } else { "not started" });
}

/// `melee golden`: the native VS match for a golden scenario (vs_match.rs).
fn golden(args: &[String]) -> ExitCode {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let iso = get("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok());
    let (Some(iso), Some(out)) = (iso, get("--out")) else {
        eprintln!("usage: melee golden --iso DISC.iso --out CANDIDATE.csv [--scenario pad_falcon_fox_fd|pad_falcon_wolf_fd] [--mods DIR [--enable id,id]] [--frames N] [--audio] [--from-snapshot FILE [--start N]] [--dump-state N:FILE]");
        return ExitCode::from(2);
    };
    let scenario = get("--scenario").unwrap_or_else(|| "pad_falcon_fox_fd".into());
    // record.sh's scene strings: stage, fighters (P1, P2), MELEE_TEST_SEED. Stocks: VS default 3
    // (the C dump shows stocks=3 in this time match; Player_GetStocks feeds the hash).
    let (stage, fighters): (&str, &[&str]) = match scenario.as_str() {
        "pad_falcon_fox_fd" => ("GrNLa.dat", &["falcon", "fox"]),
        // The same pads and stage with P2 an m-ex fighter: Akaneia's Wolf from its gdmods
        // (`mex-import import`, then --mods DIR or MELEE_MODS_DIR). No C golden exists for it.
        "pad_marth_sheik_bf" => ("GrNBa.dat", &["marth", "sheik"]),
        "pad_falcon_wolf_fd" => ("GrNLa.dat", &["falcon", "wolf"]),
        // CPU level 9 Fox (P1) v Falco (P2); the scene's "dreamland" is StKind 17 (GrGr.dat).
        "cpu_fox_falco_dl" => ("GrGr.dat", &["fox", "falco"]),
        // CPU level 9 Peach (P1) v Samus (P2) on Yoshi's Story (StKind 8), item frequency 4.
        "cpu_peach_samus_ys_items" => ("GrSt.dat", &["peach", "samus"]),
        s => {
            eprintln!("melee: scenario {s:?} not wired (pad_falcon_fox_fd)");
            return ExitCode::from(2);
        }
    };
    // The scene's `p1=fox/cpu9;p2=falco/cpu9`: slot types and levels, before the match is built.
    if scenario.starts_with("cpu_") {
        vs_match::set_cpu_player(0, 9);
        vs_match::set_cpu_player(1, 9);
    }
    let frames = get("--frames").and_then(|s| s.parse().ok());
    // gdmods over the disc: --mods DIR [--enable id,id] (else MELEE_MODS_DIR / enabled.txt).
    let mods = match get("--mods") {
        Some(dir) => {
            let en: Option<Vec<String>> = get("--enable").map(|s| s.split(',').map(str::to_string).collect());
            mods::Mods::open(std::path::Path::new(&dir), en.as_deref()).map(Some)
        }
        None => mods::Mods::from_env(),
    };
    let mods = match mods {
        Ok(m) => m,
        Err(e) => {
            eprintln!("melee: mods: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Some(m) = &mods {
        let ids: Vec<&str> = m.active.iter().map(|m| m.id.as_str()).collect();
        eprintln!("melee: mods mounted: {} ({} files)", ids.join(", "), m.files());
    }
    // `--from-snapshot FILE [--start N]`: resume from the C build's state dump (MELEE_SNAP_DUMP,
    // tools/rust-port/snap/); the CSV then holds frame N (the loaded state's own words) and on.
    let snapshot = match get("--from-snapshot").map(|p| snap::Snap::load(&p)).transpose() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("melee: {e}");
            return ExitCode::from(2);
        }
    };
    if let (Some(s), Some(n)) = (&snapshot, get("--start").and_then(|v| v.parse::<i32>().ok())) {
        if s.frame != n {
            eprintln!("melee: --start {n} but the snapshot is of frame {}", s.frame);
            return ExitCode::from(2);
        }
    }
    // `--dump-state FRAME:FILE`: write the sim's canonical state text after that frame (a resumed
    // run: right after the load), to diff a from-0 run against a resumed one.
    let dump_state: Option<(i32, String)> = get("--dump-state").and_then(|v| v.split_once(':').map(|(a, b)| (a.parse().unwrap_or(-2), b.to_string())));
    let resumed = snapshot.is_some();
    let mut from_mods = vec![];
    let res = disc::Disc::open(&iso).and_then(|mut d| {
        d.mods = mods;
        let r = match &snapshot {
            Some(s) => vs_match::VsMatch::from_snapshot(&mut d, s, stage, fighters, 12345, 3),
            None => vs_match::VsMatch::new(&mut d, stage, fighters, 12345, 3),
        };
        from_mods = std::mem::take(&mut d.from_mods);
        r
    });
    if !from_mods.is_empty() {
        eprintln!("melee: read from mods: {}", from_mods.join(" "));
    }
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
        /// A resumed run: the first row is the loaded state itself, not a step.
        resumed: bool,
        dump_state: Option<(i32, String)>,
    }
    impl SeedFrom<'_> {
        fn maybe_dump(&self) {
            if let Some((f, path)) = &self.dump_state {
                if *f == self.sim.frame {
                    if let Err(e) = std::fs::write(path, self.sim.state_dump()) {
                        eprintln!("melee: {path}: {e}");
                    }
                }
            }
        }
    }
    // `--audio`: boot the ported sound stack from the same disc and route the fighter code's
    // lbAudioAx hooks to it; the DSP runs on melee_audio::sim::virtual_frame's fixed schedule.
    if args.iter().any(|a| a == "--audio") {
        if let Err(e) = install_audio(&iso) {
            eprintln!("melee: --audio: {e}");
            return ExitCode::FAILURE;
        }
        start_match_audio(&sim, &[melee_types::CKind_Captain as i32, melee_types::CKind_Fox as i32]);
    }
    impl Sim for SeedFrom<'_> {
        fn step(&mut self) -> bool {
            if self.resumed {
                self.resumed = false;
                self.maybe_dump();
                return true;
            }
            let r = self.sim.step();
            melee_audio::sim::game_frame();
            melee_audio::sim::virtual_frame();
            if r {
                self.maybe_dump();
            }
            r
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
        let n = run(&mut SeedFrom { sim: &mut sim, golden: seed_from, resumed, dump_state }, &mut w, frames)?;
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
            eprint!("{}", sim.mex_report());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("melee: {out}: {e}");
            ExitCode::FAILURE
        }
    }
}
