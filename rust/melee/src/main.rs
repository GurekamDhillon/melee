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
//! melee golden --iso DISC.iso --out CANDIDATE.csv [--scenario pad_falcon_fox_fd|pad_falcon_wolf_fd] [--mods DIR [--enable id,id]] [--frames N] [--calltrace OUT [--calltrace-frames A-B]] [--seed-from GOLDEN.csv] [--audio]
//!              [--from-snapshot FILE.snap [--start N]] [--dump-state N:FILE]
//! melee diff --scenario NAME (--frame N | --first) [--snap FILE] [--max N] [--all]   (docs/rust-port/notes/diff.md)
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

mod crash;
mod disc;
mod mex;
mod cfg;
mod ident;
mod mods;
mod effect;
mod fighter_data;
mod stage;
mod vs_match;
mod movetab;
mod dynamics;
mod colanim;
mod gx_backend;
mod view;
mod hud_view;
mod ps_view;
mod ui;
mod retail;
mod play;
mod audio;
mod cm_drive;
mod stage_anim;
mod snap;
mod fighter_fields;
mod diff;
mod engine_view;
mod engine_audio;
mod engine_audio_gen;
mod diff_layout;
mod netmatch;
mod netplay;
mod np_env;

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

/// Debug aids, both off by default (`GUARD_ALLOC` / `LEAK_ALLOC` are read from the environment once, in `main`):
/// * `MELEE_GUARD_ALLOC=1` (or `=<min bytes>`, default 4096): every allocation of that size or more gets its own pages and ends exactly at a `PAGE_NOACCESS`
///   page, so a read past the end of a decoded table faults at once (`access violation (read of ...)` with the
///   backtrace of the reader) instead of only when the heap layout happens to put the end of a block on a page edge.
/// * `MELEE_LEAK_ALLOC=1`: `dealloc` is a no-op, to tell a use-after-free from other memory bugs.
struct DebugAlloc;
static LEAK_ALLOC: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static GUARD_ALLOC: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static GUARD_MIN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(4096);
#[inline]
fn guard_min() -> usize {
    GUARD_MIN.load(std::sync::atomic::Ordering::Relaxed)
}
#[cfg(windows)]
mod guard {
    extern "system" {
        fn VirtualAlloc(addr: *mut u8, size: usize, ty: u32, prot: u32) -> *mut u8;
        fn VirtualFree(addr: *mut u8, size: usize, ty: u32) -> i32;
        fn VirtualProtect(addr: *mut u8, size: usize, prot: u32, old: *mut u32) -> i32;
    }
    const PAGE: usize = 4096;
    /// `[head page: base address][data pages ... data ends at the guard][guard page]`; the data starts `align`ed.
    pub unsafe fn alloc(size: usize, align: usize) -> *mut u8 {
        let data_pages = (size + PAGE - 1) / PAGE;
        let total = (data_pages + 2) * PAGE;
        let base = VirtualAlloc(core::ptr::null_mut(), total, 0x3000, 0x04);
        if base.is_null() {
            return base;
        }
        let mut old = 0u32;
        VirtualProtect(base.add(total - PAGE), PAGE, 0x01, &mut old); // PAGE_NOACCESS
        let end = base.add(total - PAGE);
        let p = ((end as usize - size) & !(align.max(1) - 1)) as *mut u8;
        // remember the base just below the block (the head page is always free for it)
        (p as *mut usize).sub(1).write(base as usize);
        p
    }
    pub unsafe fn free(p: *mut u8) {
        let base = (p as *mut usize).sub(1).read() as *mut u8;
        VirtualFree(base, 0, 0x8000);
    }
}
unsafe impl std::alloc::GlobalAlloc for DebugAlloc {
    unsafe fn alloc(&self, l: std::alloc::Layout) -> *mut u8 {
        if melee_engine::arena::active() {
            let p = melee_engine::arena::alloc(l);
            if !p.is_null() {
                return p;
            }
            melee_engine::arena::note_miss(l);
        }
        #[cfg(windows)]
        if l.size() >= guard_min() && GUARD_ALLOC.load(std::sync::atomic::Ordering::Relaxed) {
            return guard::alloc(l.size(), l.align());
        }
        std::alloc::System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: std::alloc::Layout) {
        if melee_engine::arena::contains(p) {
            return melee_engine::arena::free(p, l);
        }
        if LEAK_ALLOC.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        #[cfg(windows)]
        if l.size() >= guard_min() && GUARD_ALLOC.load(std::sync::atomic::Ordering::Relaxed) {
            return guard::free(p);
        }
        std::alloc::System.dealloc(p, l)
    }
    unsafe fn alloc_zeroed(&self, l: std::alloc::Layout) -> *mut u8 {
        if melee_engine::arena::active() {
            let p = melee_engine::arena::alloc(l);
            if !p.is_null() {
                core::ptr::write_bytes(p, 0, l.size());
                return p;
            }
        }
        #[cfg(windows)]
        if l.size() >= guard_min() && GUARD_ALLOC.load(std::sync::atomic::Ordering::Relaxed) {
            return guard::alloc(l.size(), l.align()); // VirtualAlloc pages are zeroed
        }
        std::alloc::System.alloc_zeroed(l)
    }
    unsafe fn realloc(&self, p: *mut u8, l: std::alloc::Layout, n: usize) -> *mut u8 {
        if melee_engine::arena::contains(p) {
            if melee_engine::arena::same_class(l.size(), n) {
                return p;
            }
            let q = self.alloc(std::alloc::Layout::from_size_align_unchecked(n, l.align()));
            if !q.is_null() {
                core::ptr::copy_nonoverlapping(p, q, l.size().min(n));
                self.dealloc(p, l);
            }
            return q;
        }
        let guarded = |sz: usize| sz >= guard_min() && GUARD_ALLOC.load(std::sync::atomic::Ordering::Relaxed);
        if LEAK_ALLOC.load(std::sync::atomic::Ordering::Relaxed) || guarded(l.size()) || guarded(n) {
            let q = self.alloc(std::alloc::Layout::from_size_align_unchecked(n, l.align()));
            if !q.is_null() {
                core::ptr::copy_nonoverlapping(p, q, l.size().min(n));
                self.dealloc(p, l);
            }
            q
        } else {
            std::alloc::System.realloc(p, l, n)
        }
    }
}
#[global_allocator]
static GLOBAL: DebugAlloc = DebugAlloc;

fn main() -> ExitCode {
    if std::env::var_os("MELEE_LEAK_ALLOC").is_some() {
        LEAK_ALLOC.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    if let Some(v) = std::env::var_os("MELEE_GUARD_ALLOC") {
        if let Some(n) = v.to_str().and_then(|v| v.parse::<usize>().ok()).filter(|&n| n > 1) {
            GUARD_MIN.store(n, std::sync::atomic::Ordering::Relaxed);
        }
        GUARD_ALLOC.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    crash::install();
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
            // `--cpu LEVEL`: both ports CPU-controlled (a human scenario's stage and fighters).
    if let Some(l) = get("--cpu").and_then(|v| v.parse().ok()) {
        vs_match::set_cpu_player(0, l);
        vs_match::set_cpu_player(1, l);
    }
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
        Some("version") => {
            let line = std::env::current_exe().ok().and_then(|e| std::fs::read_to_string(e.with_file_name("version.txt")).ok()).and_then(|t| t.lines().next().map(str::to_string));
            println!("melee (rust) {} {}", melee_core::VERSION, line.unwrap_or_else(|| "development build".into()));
            ExitCode::SUCCESS
        }
        Some("crash-test") => crash::crash_test(args.get(1).map(String::as_str).unwrap_or("panic")),
        Some("golden") => golden(&args),
        Some("diff") => diff::diff(&args),
        Some("synctest") if use_engine(&args) => netmatch::engine_game::synctest(&args),
        Some("synctest") => netmatch::synctest(&args),
        Some("netplay") => netplay::netplay(&args),
        Some("view-golden") if use_engine(&args) => engine_view::view_golden(&args),
        Some("view-golden") => view::view_golden(&args),
        Some("play") => play::play(&args),
        Some("dat-dump") => retail::dat_dump(&args),
        Some(x) => {
            eprintln!("melee: unknown command {x:?} (selftest | hashlog | golden | diff | view-golden)");
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
        h::LBAUDIOAX_800263E8 = Some(|dir, g, beh, sfx, a, b, p0, p1, p2, s8, sc| sim::lbaudioax_800263e8(dir, g as usize, beh as i32, sfx, a, b, p0, p1, p2, s8, sc));
        h::LBAUDIOAX_800264E4 = Some(sim::lbaudioax_800264e4);
        h::LBAUDIOAX_80026510 = Some(|g| sim::lbaudioax_80026510(g as usize));
        h::LBAUDIOAX_800265C4 = Some(|g, voice| sim::lbaudioax_800265c4(g as usize, voice));
    }
    // What the positional sound objects read: the owner's class and position, the game RNG. The
    // match camera (Camera_800307D0) is not simulated yet, so an unset camera leaves them at
    // centre pan.
    sim::set_env(melee_audio::posobj::PosEnv {
        class_of: Some(|g| unsafe { (*(g as *const melee_types::HSD_GObj)).classifier as i32 }),
        fighter_pos: Some(|g| {
            let mut p = melee_types::Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            melee_fighter::ftlib::ftlib_80086644(g as *mut _, &mut p);
            [p.x, p.y, p.z]
        }),
        item_pos: Some(|g| unsafe {
            let ip = (*(g as *const melee_types::HSD_GObj)).user_data as *const melee_types::Item;
            let mut p = melee_types::Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            melee_items::itlib::it_8026b294(&*ip, &mut p);
            [p.x, p.y, p.z]
        }),
        camera: None,
        randi: Some(melee_fighter::rng::HSD_Randi),
    });
    eprintln!("melee: audio booted (smash2.sem + main.ssm), lbAudioAx hooks set");
    Ok(())
}

/// The match's audio start (lbAudioAx_8002785C + the BGM): the players' and the stage's SSM
/// banks, then the picked BGM.
/// The C build's scene-load timeline before the first logic frame (the scripted VS scene,
/// MELEE_TURBO): scene init and the bank loads at DSP sub-frame 0, the BGM at sub-frame 10 (3
/// video frames in), the first logic frame (Ready call) at sub-frame 106 (29 frames later). The
/// mix of those sub-frames is appended to `pre` when a capture is on.
fn start_match_audio(sim: &vs_match::VsMatch, ckinds: &[i32], stage: &str, preroll: u32, mut pre: Option<&mut Vec<i16>>) {
    let (stkind, grkind) = audio::stage_kinds(stage);
    let mut frames = |n: u32, pre: &mut Option<&mut Vec<i16>>| {
        for _ in 0..n {
            melee_audio::sim::game_frame();
            match pre.as_deref_mut() {
                Some(w) => {
                    melee_audio::sim::virtual_frame_into(w);
                }
                None => {
                    melee_audio::sim::virtual_frame();
                }
            }
        }
    };
    melee_audio::sim::scene_init();
    melee_audio::sim::load_announcer();
    melee_audio::sim::load_match_banks(ckinds, stkind, grkind);
    frames(3, &mut pre);
    let r = melee_audio::sim::play_bgm_id(sim.bgm);
    frames(preroll, &mut pre);
    let loaded = melee_audio::sim::with(|s| s.lb.state.iter().enumerate().filter(|(_, &x)| x == 2).map(|(i, _)| melee_audio::lbaudio::SSM_FILES[i]).collect::<Vec<_>>());
    eprintln!("melee: match audio: banks {:?}, bgm {} ({}) {}", loaded.unwrap_or_default(), sim.bgm,
        melee_audio::lbaudio::HPS_FILES.get(sim.bgm as usize).unwrap_or(&"?"), if r == 0 { "started" } else { "not started" });
}

/// A 32 kHz s16 stereo WAV, like `shim_ax.c`'s `MELEE_AUDIO_DUMP`.
pub(crate) fn write_wav(path: &str, samples: &[i16]) -> std::io::Result<()> {
    let bytes = (samples.len() * 2) as u32;
    let mut b = Vec::with_capacity(44 + bytes as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + bytes).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&32000u32.to_le_bytes());
    b.extend_from_slice(&(32000u32 * 4).to_le_bytes());
    b.extend_from_slice(&4u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&bytes.to_le_bytes());
    for s in samples {
        b.extend_from_slice(&s.to_le_bytes());
    }
    std::fs::write(path, b)
}

/// The engine booted into a golden scenario's match (scene string, disc, pad feed); `melee golden --engine tu` and `melee diff --engine tu`.
pub(crate) fn boot_engine_scenario(iso: &str, scenario: &str) -> Result<melee_engine::Engine, String> {
    let scene = match scenario {
        "pad_falcon_fox_fd" => "p1=falcon/human;p2=fox/human;p3=none;p4=none;stage=fd;match=time;minutes=0;items=off",
        "pad_marth_sheik_bf" => "p1=marth/human;p2=sheik/human;p3=none;p4=none;stage=bf;match=time;minutes=0;items=off",
        "cpu_fox_falco_dl" => "p1=fox/cpu9;p2=falco/cpu9;p3=none;p4=none;stage=dreamland;match=time;minutes=0;items=off",
        "cpu_peach_samus_ys_items" => "p1=peach/cpu9;p2=samus/cpu9;p3=none;p4=none;stage=ys;match=time;minutes=0;items=4",
        s => return Err(format!("scenario {s:?} not wired")),
    };
    // `--time-secs 60`: a one-minute time match (the audio proof's clock)
    let scene = match std::env::var("MELEE_ENGINE_MINUTES") {
        Ok(m) => scene.replace("minutes=0", &format!("minutes={m}")),
        Err(_) => scene.to_string(),
    };
    let scene = scene.as_str();
    // draws the C boot makes that the engine's boot flow does not yet (tools/rust-port/engine/pre_draws.tsv; shrinking to empty)
    if std::env::var_os("MELEE_ENGINE_PRE_DRAWS").is_none() {
        let n = std::fs::read_to_string("tools/rust-port/engine/pre_draws.tsv").ok()
            .and_then(|t| t.lines().find_map(|l| l.strip_prefix(&format!("{scenario}	")).map(str::to_string)))
            .unwrap_or_else(|| if scenario.starts_with("pad_") { "1".into() } else { "0".into() });
        // SAFETY: single-threaded start-up, before the engine reads it.
        unsafe { std::env::set_var("MELEE_ENGINE_PRE_DRAWS", n.trim()) };
    }
    let disc = engine_disc(iso)?;
    let scn = melee_engine::scenario::Scenario::parse(&format!("mode=vs;at=match;{scene}")).expect("scene");
    let mut e = melee_engine::Engine::boot_vs(disc, 64 << 20, melee_platform::PadSystem::headless(), scn).map_err(|e| format!("engine boot: {e}"))?;
    // the scenario's pad script (pad_fixed.lua): golden frame k reads sample k-1 (a fixed one-frame latency), as the legacy path does
    if scenario.starts_with("pad_") {
        e.set_pad_feed(|tick, port| {
            let r = vs_match::input::pad_fixed(port, tick as i32);
            let mut st = melee_types::PADStatus::zeroed();
            st.button = r.button as u16;
            st.stickX = r.stick_x;
            st.stickY = r.stick_y;
            st.substickX = r.sub_x;
            st.substickY = r.sub_y;
            st.triggerLeft = r.analog_l;
            st.triggerRight = r.analog_r;
            // the scripted match has two controllers: ports 3 and 4 report PAD_ERR_NO_CONTROLLER (the controller indicator's sound reads it)
            if port >= 2 { st.err = -1; }
            st
        });
    }
    Ok(e)
}

/// Step 18 (cutover): the consolidated engine is the default. `--engine legacy` selects the `vs_match` model, and so does
/// anything only the model has (mods, replays, snapshot resume, audio, an unwired scenario).
pub(crate) fn use_engine(args: &[String]) -> bool {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    match get("--engine").as_deref() {
        Some("legacy") => return false,
        Some("tu") => return true,
        _ => {}
    }
    const LEGACY_ONLY: [&str; 3] = ["--from-snapshot", "--dump-state", "--start"];
    if args.iter().any(|a| LEGACY_ONLY.contains(&a.as_str())) {
        return false;
    }
    matches!(get("--scenario").as_deref().unwrap_or("pad_falcon_fox_fd"), "pad_falcon_fox_fd" | "pad_marth_sheik_bf" | "cpu_fox_falco_dl" | "cpu_peach_samus_ys_items")
}

static mut REPLAY: Option<melee_data::player::Player> = None;

static mut PENDING_OVERLAY: Option<melee_engine::gcm::Overlay> = None;

/// The disc image for an engine boot, with the pending mods overlay (`set_engine_mods`) mounted over it.
pub(crate) fn engine_disc(iso: &str) -> Result<melee_engine::gcm::Gcm, String> {
    let mut d = melee_engine::gcm::Gcm::open(iso)?;
    if let Some(o) = unsafe { (*std::ptr::addr_of_mut!(PENDING_OVERLAY)).take() } {
        d.set_overlay(o);
    }
    Ok(d)
}

/// Mount the `--mods` / settings mods over the next engine boot.
pub(crate) fn set_engine_mods(args: &[String]) -> Result<(), String> {
    unsafe { PENDING_OVERLAY = engine_mods_overlay(args)? };
    Ok(())
}

/// `--mods DIR [--enable id,id]` (else `MELEE_MODS_DIR` / settings): the gdmods mounted over the disc, as a `Gcm` overlay.
pub(crate) fn engine_mods_overlay(args: &[String]) -> Result<Option<melee_engine::gcm::Overlay>, String> {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let mods = match get("--mods") {
        Some(dir) => {
            let en: Option<Vec<String>> = get("--enable").map(|s| s.split(',').map(str::to_string).collect());
            mods::Mods::open(std::path::Path::new(&dir), en.as_deref()).map(Some)?
        }
        None => mods::Mods::from_env()?,
    };
    let Some(m) = mods else { return Ok(None) };
    let ids: Vec<&str> = m.active.iter().map(|m| m.id.as_str()).collect();
    eprintln!("melee: mods mounted: {} ({} files)", ids.join(", "), m.files());
    Ok(Some(Box::new(move |path: &str| m.lookup(path).and_then(|p| std::fs::read(p).ok()))))
}

/// Route the fighter code's `Replay_*` platform hooks (gw_replay.c) to `pl`: the recorded inputs of the frame being simulated.
fn install_replay(pl: melee_data::player::Player) {
    use melee_fighter::platform::hook as h;
    unsafe {
        REPLAY = Some(pl);
        let r = || REPLAY.as_mut().unwrap();
        h::Replay_Active = Some(|| 1);
        h::Replay_HasInput = Some(|p, f| unsafe { r_in().has_input(p, f != 0) as i32 });
        h::Replay_StickX = Some(|p, f| unsafe { r_in().input(p, f != 0).map_or(0.0, |i| i.lx) });
        h::Replay_StickY = Some(|p, f| unsafe { r_in().input(p, f != 0).map_or(0.0, |i| i.ly) });
        h::Replay_CStickX = Some(|p, f| unsafe { r_in().input(p, f != 0).map_or(0.0, |i| i.cx) });
        h::Replay_CStickY = Some(|p, f| unsafe { r_in().input(p, f != 0).map_or(0.0, |i| i.cy) });
        h::Replay_Trigger = Some(|p, f| unsafe { r_in().input(p, f != 0).map_or(0.0, |i| i.trigger) });
        h::Replay_Buttons = Some(|p, f| unsafe { r_in().input(p, f != 0).map_or(0, |i| i.buttons) });
        h::Replay_UcfCardinals = Some(|p| unsafe { r_in().ucf_cardinals(p) as i32 });
        h::Replay_UcfVersion = Some(|p| unsafe { r_in().ucf_version(p) });
        h::Replay_RawStick = Some(|p, w| unsafe { r_in().raw_stick(p, w) });
        h::Replay_RawStickBack = Some(|p, w, b| unsafe { r_in().raw_stick_back(p, w, b) });
        h::Replay_ChecksumSeed = Some(|arrived, frame| unsafe { r_in().checksum_seed(arrived, frame) });
        let _ = r;
    }
}

#[allow(static_mut_refs)]
unsafe fn r_in() -> &'static mut melee_data::player::Player {
    REPLAY.as_mut().unwrap()
}

/// `melee golden --engine tu`: the same CSV from the consolidated engine (melee_engine): the real StartMelee path,
/// the TU-ported game code over native structs, `RB_NeutralHash` (NHV1) after each scene tick. `--frames N`
/// (default 1) ticks; row k is the state after tick k (frame 0 = the first tick).
fn golden_engine(args: &[String]) -> ExitCode {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let iso = get("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok());
    let (Some(iso), Some(out)) = (iso, get("--out")) else {
        eprintln!("usage: melee golden --engine tu --iso DISC.iso --out CANDIDATE.csv [--scenario NAME] [--frames N]");
        return ExitCode::from(2);
    };
    let scenario = get("--scenario").unwrap_or_else(|| "pad_falcon_fox_fd".into());
    if let Err(er) = set_engine_mods(args) {
        eprintln!("melee: mods: {er}");
        return ExitCode::FAILURE;
    }
    // default: the golden's length (the pad scripts end at 1801 frames)
    let frames: usize = get("--frames").and_then(|s| s.parse().ok()).unwrap_or(if scenario.starts_with("pad_") { 1802 } else { 3601 });
    // `--shots 60,300 [--shot-dir DIR]`: render those frames offscreen (melee_gx on wgpu, no window) to PNG
    let shots: Vec<usize> = get("--shots").map(|s| s.split(',').filter_map(|x| x.trim().parse().ok()).collect()).unwrap_or_default();
    let shot_dir = std::path::PathBuf::from(get("--shot-dir").unwrap_or_else(|| "target/shots".into()));
    let mut renderer: Option<melee_gx::render::Renderer> = None;
    if !shots.is_empty() {
        unsafe { melee_engine::engine::RENDER = true };
        let _ = std::fs::create_dir_all(&shot_dir);
    }
    // `--audio [--audio-wav OUT] [--audio-trace OUT] [--audio-preroll N]`: the sound stack from the same disc, driven by the game's own lbAudioAx calls
    let audio = args.iter().any(|a| a == "--audio");
    if audio {
        // the C audio captures ran headless (turbo, no render pass): the draw callbacks have sound side effects (the off-screen arrow's)
        // SAFETY: single-threaded start-up, before the engine reads it.
        if std::env::var_os("MELEE_ENGINE_RENDER_AUDIO").is_none() { unsafe { std::env::set_var("MELEE_ENGINE_HEADLESS", "1") }; }
        if let Some(secs) = get("--time-secs").and_then(|v| v.parse::<i32>().ok()).filter(|&s| s < 180) {
            // SAFETY: single-threaded start-up, before the engine reads it.
            unsafe { std::env::set_var("MELEE_ENGINE_MINUTES", ((secs + 59) / 60).to_string()) };
        }
        // SAFETY: single-threaded start-up, before the engine reads it.
        unsafe { std::env::set_var("MELEE_ENGINE_PRE_DRAWS", "0") }; // the BGM pick below makes the draw the dummy stood for
        if let Err(er) = engine_audio::install(&iso, get("--audio-trace").is_some()) {
            eprintln!("melee: --audio: {er}");
            return ExitCode::FAILURE;
        }
        if get("--audio-wav").is_some() {
            unsafe { engine_audio::WAV = Some(Vec::new()) };
        }
    }
    // `--slp FILE` / `--gdlp FILE`: a Slippi / whole-match replay drives the match (stage, fighters, seed and every frame's inputs and RNG seed)
    let mut replay = false;
    let mut e = if get("--slp").is_some() || get("--gdlp").is_some() {
        let gd = get("--gdlp");
        let ident = gd.as_ref().map(|_| {
            let exe = std::env::current_exe().unwrap_or_default();
            ident::identity(&exe, std::path::Path::new(&iso), "")
        });
        let r = match (&gd, get("--slp")) {
            (Some(g), _) => slp_open(g, ident),
            (None, Some(p)) => slp_open(&p, None),
            _ => unreachable!(),
        };
        match r {
            Ok((pl, stage, names, seed)) => {
                let key = match stage { "GrNLa.dat" => "fd", _ => "bf" };
                let scene = format!("mode=vs;at=match;p1={}/human;p2={}/human;p3=none;p4=none;stage={key};match=time;minutes=0;items=off", names[0], names[1]);
                // SAFETY: single-threaded start-up, before the engine reads them.
                unsafe {
                    std::env::set_var("MELEE_TEST_SEED", seed.to_string());
                    if std::env::var_os("MELEE_ENGINE_PRE_DRAWS").is_none() { std::env::set_var("MELEE_ENGINE_PRE_DRAWS", "1"); }
                }
                let booted = engine_disc(&iso).and_then(|disc| {
                    let scn = melee_engine::scenario::Scenario::parse(&scene)?;
                    melee_engine::Engine::boot_vs(disc, 64 << 20, melee_platform::PadSystem::headless(), scn)
                });
                install_replay(pl);
                replay = true;
                booted
            }
            Err(er) => Err(er),
        }
    } else {
        boot_engine_scenario(&iso, &scenario)
    };
    let mut e = match e {
        Ok(e) => e,
        Err(er) => {
            eprintln!("melee: {er}");
            return ExitCode::FAILURE;
        }
    };
    let frames = if replay && get("--frames").is_none() { 1802 } else { frames };
    if audio {
        engine_audio::start_stage_bgm();
        // the C's load: the first logic frame comes this many video frames after the BGM starts
        for _ in 0..get("--audio-preroll").and_then(|v| v.parse::<u32>().ok()).unwrap_or(29) {
            engine_audio::load_frame();
        }
    }
    let idle = args.iter().any(|a| a == "--idle-pads");
    // `--pause F:T`: START after logic frame F pauses, the pauser's START resumes at T (the audio keeps running on the held frames)
    let pause: Option<(usize, usize)> = get("--pause").and_then(|v| v.split_once(':').and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?))));
    let mut csv = String::from("frame,hash,seed,items,itemsum,s0,s1,s2,s3,s4,s5,s6,s7,s8,s9,s10,s11\n");
    let calltrace_out = get("--calltrace");
    if calltrace_out.is_some() && !melee_core::ct::ENABLED {
        eprintln!("melee: --calltrace needs a build with `--features calltrace`");
        return ExitCode::from(2);
    }
    if let Some((a, b)) = get("--calltrace-frames").and_then(|v| v.split_once('-').map(|(a, b)| (a.to_string(), b.to_string()))) {
        if let (Ok(a), Ok(b)) = (a.parse::<i64>(), b.parse::<i64>()) {
            melee_core::ct::set_range(a, b);
        }
    }
    for f in 0..frames {
        melee_core::ct::begin_frame(f as i64);
        if replay {
            // gmscene.c: `Replay_Tick`, then the forced seed; the fighters read the recorded inputs through the `Replay_*` platform hooks
            let pl = unsafe { REPLAY.as_mut().unwrap() };
            pl.tick();
            let sd = pl.resync_seed();
            if sd != 0 {
                melee_runtime::tu::random::global::state().set_logic_seed(sd);
            }
        }
        if idle || pause.is_some() {
            let mut pads = [melee_types::PADStatus::zeroed(); 4];
            pads[2].err = -1;
            pads[3].err = -1;
            if pause.is_some_and(|(a, b)| f == a + 1 || f == b + 1) {
                pads[0].button = 0x1000; // START
            }
            e.set_pads(Some(pads));
        }
        if !e.tick() {
            break;
        }
        if audio {
            engine_audio::dsp_frame();
        }
        if std::env::var_os("MELEE_PAD_DBG").is_some() {
            let q = unsafe { &melee_runtime::tu::controller::HSD_PadGameStatus[0] };
            eprintln!("PADDBG tick {f} btn {:X} sx {} sy {} nx {} ny {}", q.button, q.stickX, q.stickY, q.nml_stickX, q.nml_stickY);
        }
        if !shots.is_empty() {
            let frame = melee_engine::gxbridge::take_frame();
            if shots.contains(&f) {
                match frame {
                    Some(draws) => {
                        let rd = renderer.get_or_insert_with(|| melee_gx::render::Renderer::new(640, 480).expect("renderer"));
                        let px = rd.render(&melee_engine::gxbridge::shim().gx.state, &draws);
                        let path = shot_dir.join(format!("engine_{scenario}_{f:04}.png"));
                        match melee_platform::gfx::write_png(&path, rd.width, rd.height, &px) {
                            Ok(()) => eprintln!("melee: frame {f}: {} draws -> {}", draws.len(), path.display()),
                            Err(er) => eprintln!("melee: screenshot: {er}"),
                        }
                    }
                    None => eprintln!("melee: frame {f}: no GX frame was finished (GXCopyDisp not reached)"),
                }
            }
        }
        let r = e.neutral_row();
        if let Ok(spec) = std::env::var("MELEE_ENGINE_DUMP") {
            // MELEE_ENGINE_DUMP=FRAME:FILE: the Fighter records in the legacy `--dump-state` form ("p1 fp.NAME HEX"), to diff by field.
            if let Some((n, file)) = spec.split_once(':') {
                if n.parse::<usize>().ok() == Some(f) {
                    let mut t = String::new();
                    t.push_str(&e.debug_tree(0));
                    t.push_str(&e.debug_map_gobjs());
                    t.push_str(&e.debug_items());
                    for slot in 0..2 {
                        if let Some(b) = e.fighter_bytes(slot) {
                            let mut fl: Vec<(&str, usize)> = fighter_fields::FIGHTER_FIELDS.to_vec();
                            fl.sort_by_key(|x| x.1);
                            for (k, (name, off)) in fl.iter().enumerate() {
                                let end = fl.get(k + 1).map_or(b.len(), |x| x.1);
                                t.push_str(&format!("p{} fp.{name} ", slot + 1));
                                for x in &b[*off..end] { t.push_str(&format!("{x:02x}")); }
                                t.push('\n');
                            }
                        }
                    }
                    let _ = std::fs::write(file, t);
                }
            }
        }
        if std::env::var_os("MELEE_ENGINE_ITEMS").is_some() {
            for l in e.debug_items().lines() { eprintln!("f{f} {l}"); }
        }
        if std::env::var_os("MELEE_ENGINE_HURT").is_some() {
            for slot in 0..2 {
                for l in e.debug_hurt(slot).lines() { eprintln!("f{f} p{} {l}", slot + 1); }
            }
        }
        if std::env::var_os("MELEE_ENGINE_FT").is_some() {
            eprintln!("f{f} {}", e.debug_fighter(0));
            eprintln!("f{f} {}", e.debug_fighter(1));
        }
        csv.push_str(&format!("{f},{:08X},{:08X},{},{:08X}", r[0], r[1], r[2], r[3]));
        for w in &r[4..16] {
            csv.push_str(&format!(",{w:08X}"));
        }
        csv.push('\n');
        // written every frame: a crash later in the run keeps the frames that were produced
        let _ = std::fs::write(&out, &csv);
    }
    if let Some(p) = &calltrace_out {
        match melee_core::ct::write(p) {
            Ok(n) => eprintln!("melee: {n} calls written to {p}"),
            Err(er) => eprintln!("melee: {p}: {er}"),
        }
    }
    if audio {
        if let Some(p) = get("--audio-trace") {
            let t = melee_audio::sim::with(|s| s.trace.take()).flatten().unwrap_or_default();
            let _ = std::fs::write(&p, t.join("
") + "
");
            eprintln!("melee: {} audio calls written to {p}", t.len());
        }
        if let (Some(p), Some(w)) = (get("--audio-wav"), unsafe { (*std::ptr::addr_of_mut!(engine_audio::WAV)).take() }) {
            if let Err(er) = write_wav(&p, &w) {
                eprintln!("melee: {p}: {er}");
            }
            eprintln!("melee: audio mix of {} sub-frames written to {p}", w.len() / 320);
        }
    }
    if let Err(er) = std::fs::write(&out, csv) {
        eprintln!("melee: {out}: {er}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// `melee golden`: the native VS match for a golden scenario (vs_match.rs).
fn golden(args: &[String]) -> ExitCode {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    if use_engine(args) {
        return golden_engine(args);
    }
    let iso = get("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok());
    let (Some(iso), Some(out)) = (iso, get("--out")) else {
        eprintln!("usage: melee golden --iso DISC.iso --out CANDIDATE.csv [--scenario pad_falcon_fox_fd|pad_falcon_wolf_fd] [--mods DIR [--enable id,id]] [--frames N] [--audio [--audio-wav OUT.wav]] [--from-snapshot FILE [--start N]] [--dump-state N:FILE]");
        return ExitCode::from(2);
    };
    crash::note_disc(&iso);
    let scenario = get("--scenario").unwrap_or_else(|| "pad_falcon_fox_fd".into());
    // gdmods over the disc: --mods DIR [--enable id,id] (else MELEE_MODS_DIR / enabled.txt).
    let mods = match get("--mods") {
        Some(dir) => {
            let en: Option<Vec<String>> = get("--enable").map(|s| s.split(',').map(str::to_string).collect());
            mods::Mods::open(std::path::Path::new(&dir), en.as_deref()).map(Some)
        }
        None => mods::Mods::from_env(),
    };
    let mut mods = match mods {
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
    // `--slp FILE`: play a Slippi/GDLP replay (the C's MELEE_SLP=FILE with MELEE_SLP_RESYNC=1): the
    // stage, fighters and seed come from the file, every frame's inputs and RNG seed from its events.
    let slp_path = get("--slp");
    let gdlp_path = get("--gdlp");
    // `--gdlp FILE`: a whole-match recording plays only on the build / disc / mods / Geno data it was
    // made with (MELEE_GDLP, gw_Replay_LoadGdlp): the identity of THIS binary is checked first.
    let ident = gdlp_path.as_ref().map(|_| {
        let exe = std::env::current_exe().unwrap_or_default();
        let d = mods.as_mut().map(|m| m.set.describe()).unwrap_or_default();
        ident::identity(&exe, std::path::Path::new(&iso), &d)
    });
    let slp: Option<(melee_data::player::Player, &'static str, [&'static str; 2], u32)> = match (slp_path, gdlp_path) {
        (None, None) => None,
        (p, g) => {
            let r = match (&g, &p) {
                (Some(g), _) => slp_open(g, ident),
                (None, Some(p)) => slp_open(p, None),
                _ => unreachable!(),
            };
            match r {
                Ok(x) => Some(x),
                Err(e) => {
                    eprintln!("melee: --slp/--gdlp: {e}");
                    return ExitCode::from(2);
                }
            }
        }
    };
    // record.sh's scene strings: stage, fighters (P1, P2), MELEE_TEST_SEED. Stocks: VS default 3
    // (the C dump shows stocks=3 in this time match; Player_GetStocks feeds the hash).
    let slp_fighters: Option<[&str; 2]> = slp.as_ref().map(|x| x.2);
    let (stage, fighters): (&str, &[&str]) = if let Some(x) = &slp {
        (x.1, slp_fighters.as_ref().map(|f| &f[..]).unwrap())
    } else {
        match scenario.as_str() {
        "pad_falcon_fox_fd" => ("GrNLa.dat", &["falcon", "fox"]),
        // The same pads and stage with P2 an m-ex fighter: Akaneia's Wolf from its gdmods
        // (`mex-import import`, then --mods DIR or MELEE_MODS_DIR). No C golden exists for it.
        "pad_marth_sheik_bf" => ("GrNBa.dat", &["marth", "sheik"]),
        "pad_falcon_wolf_fd" => ("GrNLa.dat", &["falcon", "wolf"]),
        // CPU level 9 Fox (P1) v Falco (P2); the scene's "dreamland" is StKind 17 (GrGr.dat).
        // The audio proof's time-limit scenario: the same CPUs on Final Destination.
        "cpu_fox_falco_fd" => ("GrNLa.dat", &["fox/cpu9", "falco/cpu9"]),
        "cpu_fox_falco_dl" => ("GrGr.dat", &["fox/cpu9", "falco/cpu9"]),
        // CPU level 9 Peach (P1) v Samus (P2) on Yoshi's Story (StKind 8), item frequency 4.
        "cpu_peach_samus_ys_items" => ("GrSt.dat", &["peach/cpu9", "samus/cpu9"]),
        s => {
            eprintln!("melee: scenario {s:?} not wired (pad_falcon_fox_fd)");
            return ExitCode::from(2);
        }
        }
    };
    // The scene's `p1=fox/cpu9;p2=falco/cpu9`: slot types and levels, before the match is built.
    if scenario.starts_with("cpu_") {
        vs_match::set_cpu_player(0, 9);
        vs_match::set_cpu_player(1, 9);
    }
    // The pad goldens are 1801 frames long (the scripts end there); without a cap pad_marth_sheik_bf
    // keeps simulating until the caller's timeout.
    let frames = get("--frames").and_then(|s| s.parse().ok()).or(match scenario.as_str() {
        "pad_falcon_fox_fd" | "pad_marth_sheik_bf" if slp.is_none() => Some(1802),
        _ => None,
    });
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
    let slp_seed = slp.as_ref().map_or(12345, |x| x.3);
    let mut from_mods = vec![];
    let res = disc::Disc::open(&iso).and_then(|mut d| {
        d.mods = mods;
        let item_freq = if scenario == "cpu_peach_samus_ys_items" { Some(4) } else { None };
        let r = match &snapshot {
            Some(s) => vs_match::VsMatch::from_snapshot(&mut d, s, stage, fighters, 12345, 3),
            None => vs_match::VsMatch::new_ex(&mut d, stage, fighters, slp_seed, 3, item_freq),
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
        /// `--audio-wav`: every sub-frame's s16 stereo mix.
        wav: Option<Vec<i16>>,
        /// `--slp`: the replay driving the match.
        replay: Option<melee_data::player::Player>,
        /// `--pause F:T`: START pauses after logic frame F and the pauser's START resumes T-F
        /// frames later (the audio keeps running on the held frames, as in the C build).
        pause: Option<(i32, i32)>,
        /// `--time-secs`: the match clock the timer callouts count down to.
        time_secs: i32,
        /// Frames since the clock ran out (`--time-secs` under 180).
        halted: Option<i32>,
    }
    impl SeedFrom<'_> {
        fn audio_frame(&mut self) {
            melee_audio::sim::game_frame();
            match &mut self.wav {
                Some(w) => {
                    melee_audio::sim::virtual_frame_into(w);
                }
                None => {
                    melee_audio::sim::virtual_frame();
                }
            }
        }
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
    // `--idle-pads`: nobody touches a pad (the golden corpus's CPU/idle pad scripts, pad_pause.lua).
    if args.iter().any(|a| a == "--idle-pads") {
        sim.pad_feed = Some([Default::default(); 4]);
    }
    let mut pre_wav: Vec<i16> = Vec::new();
    // `--audio`: boot the ported sound stack from the same disc and route the fighter code's
    // lbAudioAx hooks to it; the DSP runs on melee_audio::sim::virtual_frame's fixed schedule.
    if args.iter().any(|a| a == "--audio") {
        if let Err(e) = install_audio(&iso) {
            eprintln!("melee: --audio: {e}");
            return ExitCode::FAILURE;
        }
        if get("--audio-trace").is_some() {
            melee_audio::sim::with(|s| s.trace = Some(Vec::new()));
        }
        let ckinds: Vec<i32> = fighters.iter().map(|f| audio::ckind_of(f)).collect();
        let mut v = Vec::new();
        start_match_audio(&sim, &ckinds, stage, get("--audio-preroll").and_then(|v| v.parse().ok()).unwrap_or(29), if get("--audio-wav").is_some() { Some(&mut v) } else { None });
        pre_wav = v;
    }
    impl Sim for SeedFrom<'_> {
        fn step(&mut self) -> bool {
            if self.resumed {
                self.resumed = false;
                self.maybe_dump();
                return true;
            }
            if let Some(pl) = &mut self.replay {
                slp_frame(pl, self.sim);
            }
            if let Some(h) = self.halted {
                // The clock ran out: the sim holds, the sound goes on (TIME!, then the scene change).
                self.halted = Some(h + 1);
                if h == 0 {
                    // gmVs: the end of the clock stops every track before the call.
                    audio::Audio::detached().match_leave();
                }
                if h + 1 == audio::TIME_CALL_DELAY {
                    audio::Audio::detached().match_end_call(true);
                }
                self.audio_frame();
                return h < 100;
            }
            melee_core::ct::begin_frame(self.sim.frame as i64 + 1);
            let r = self.sim.step();
            if r && self.time_secs < 180 && self.sim.frame == audio::clock_end(self.sim.go_frame_for_hud(), self.time_secs) - 1 {
                self.halted = Some(0);
            }
            if r && melee_audio::sim::is_installed() {
                audio::match_frame_calls(self.sim.frame, self.sim.go_frame_for_hud(), self.time_secs);
            }
            self.audio_frame();
            if r && self.pause.is_some_and(|(f, _)| f == self.sim.frame) {
                let (f, t) = self.pause.take().unwrap();
                let a = audio::Audio::detached();
                a.pause_start();
                for _ in f..t {
                    self.audio_frame();
                }
                a.pause_end();
                // un_802FDA78 plays the controller indicator's sound two frames after the resume.
                for _ in 0..2 {
                    self.audio_frame();
                }
                a.pause_resume_sound();
            }
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
    // `--calltrace OUT [--calltrace-frames A-B]`: the ordered TU call list per frame (needs `--features calltrace`;
    // tools/rust-port/trace/seqdiff.py compares it with the C build's MELEE_CALLTRACE_SEQ).
    let calltrace_out = get("--calltrace");
    if calltrace_out.is_some() {
        if !melee_core::ct::ENABLED {
            eprintln!("melee: --calltrace needs a build with `--features calltrace` (cargo build -p melee --release --features calltrace)");
            return ExitCode::from(2);
        }
        if let Some((a, b)) = get("--calltrace-frames").and_then(|v| v.split_once('-').map(|(a, b)| (a.to_string(), b.to_string()))) {
            if let (Ok(a), Ok(b)) = (a.parse::<i64>(), b.parse::<i64>()) {
                melee_core::ct::set_range(a, b);
            }
        }
    }
    let wav_path = get("--audio-wav");
    let trace_path = get("--audio-trace");
    let injected = seed_from.is_some();
    let res = std::fs::File::create(&out).and_then(|f| {
        let mut w = std::io::BufWriter::new(f);
        let mut sf = SeedFrom { sim: &mut sim, golden: seed_from, resumed, dump_state, halted: None, time_secs: get("--time-secs").and_then(|v| v.parse().ok()).unwrap_or(180), pause: get("--pause").and_then(|v| v.split_once(':').and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?)))), wav: wav_path.as_ref().map(|_| std::mem::take(&mut pre_wav)), replay: slp.map(|x| x.0) };
        let n = run(&mut sf, &mut w, frames)?;
        w.flush()?;
        if let Some(p) = &trace_path {
            let t = melee_audio::sim::with(|s| s.trace.take()).flatten().unwrap_or_default();
            std::fs::write(p, t.join("\n") + "\n")?;
            eprintln!("melee: {} audio calls written to {p}", t.len());
        }
        if let (Some(p), Some(samples)) = (&wav_path, &sf.wav) {
            write_wav(p, samples)?;
            eprintln!("melee: audio mix of {} sub-frames written to {p}", samples.len() / 320);
        }
        Ok(n)
    });
    if let Some(p) = &calltrace_out {
        match melee_core::ct::write(p) {
            Ok(n) => eprintln!("melee: {n} calls written to {p}"),
            Err(e) => eprintln!("melee: {p}: {e}"),
        }
    }
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

/// Load a replay for `golden --slp`: the player, the stage file, the two fighters and the seed.
/// Only what the native sim plays is accepted (Falcon, Fox, Marth, Sheik on FD / Battlefield).
fn slp_open(path: &str, gdlp: Option<melee_data::gdlp::Identity>) -> Result<(melee_data::player::Player, &'static str, [&'static str; 2], u32), String> {
    let data = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    let mut pl = melee_data::player::Player::new(melee_data::player::Env { resync: true, identity: gdlp, ..Default::default() });
    if gdlp.is_some() {
        pl.load_gdlp(path, &data, false);
    } else {
        pl.load_slp(path, &data, false);
    }
    for l in pl.log.drain(..) {
        eprintln!("melee: {l}");
    }
    if !pl.active {
        return Err(format!("{path}: the replay was refused"));
    }
    // gw_Replay_ApplyMatch arms the counter: the first Replay_Tick makes it -123.
    pl.frame = -124;
    // "mode=vs;at=match;stage=ext:32;p1=ck:0/c0/hu;p2=ck:2/c1/hu"
    let scene = pl.r.scene.clone();
    let mut stage = None;
    let mut fighters: Vec<&'static str> = Vec::new();
    for part in scene.split(';') {
        if let Some(e) = part.strip_prefix("stage=ext:") {
            stage = match e {
                "32" => Some("GrNLa.dat"),
                "31" => Some("GrNBa.dat"),
                _ => return Err(format!("stage ext:{e} is not playable in the Rust sim yet")),
            };
        } else if part.starts_with('p') && part.contains("=ck:") {
            let ck = part.split("ck:").nth(1).and_then(|r| r.split('/').next()).unwrap_or("");
            if !part.ends_with("/hu") {
                return Err(format!("{part}: CPU players are not replayed"));
            }
            fighters.push(match ck {
                "0" => "falcon",
                "2" => "fox",
                "9" => "marth",
                "19" => "sheik",
                _ => return Err(format!("fighter ck:{ck} is not playable in the Rust sim yet")),
            });
        }
    }
    let Some(stage) = stage else {
        return Err("the replay has no stage".into());
    };
    let [a, b]: [&'static str; 2] = fighters.try_into().map_err(|_| "the replay is not a two-player match".to_string())?;
    let seed = pl.r.seed;
    Ok((pl, stage, [a, b], seed))
}

/// One frame of replay playback before the sim steps (gmscene.c: `Replay_Tick`, then the forced seed;
/// fighter.c: the recorded inputs): the frame being simulated is the replay's `pl.frame`.
fn slp_frame(pl: &mut melee_data::player::Player, sim: &mut vs_match::VsMatch) {
    use melee_net::pad::RbInput;
    pl.tick();
    let s = pl.resync_seed();
    if s != 0 {
        sim.rb_force_seed(s);
    }
    let mut ins: [Option<RbInput>; 4] = [None; 4];
    for (port, slot) in ins.iter_mut().enumerate() {
        *slot = pl.input(port as i32, false).map(|i| RbInput {
            lx: i.lx,
            ly: i.ly,
            cx: i.cx,
            cy: i.cy,
            trigger: i.trigger,
            buttons: i.buttons,
            seed: i.seed,
            raw: i.raw,
            present: 1,
            ..Default::default()
        });
    }
    sim.replay_in = Some(ins);
    sim.pad_feed = Some([vs_match::input::RawPad::default(); 4]);
}
