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

mod geno_suite;
mod cosmetics;
mod cosmetics_import;
mod nucleus;
mod settings_overlay;
mod crash;
mod launcher_compat;
mod display;
mod prof;
mod heapprof;
mod disc;
mod cfg;
mod ident;
mod mods;
mod gx_backend;
mod ui;
mod retail;
mod play;
#[cfg(feature = "corev1")]
mod corev1;
mod retail_flow;
mod audio;
mod stage_anim;
mod slpcheck;
mod snap;
mod fighter_fields;
mod diff;
mod engine_view;
mod engine_audio;
mod engine_audio_gen;
mod diff_layout;
mod dolphin_oracle;
mod netmatch;
mod netplay;
mod slippi_np;
mod play_net;
mod slp_export;
mod spectate;
mod replay_ctl;
mod replays;
mod np_lockstep;
mod np_mux;
mod np_early;
mod np_env;
mod np_hud;
mod input_delay;
mod menu_support;
mod pads;
mod practice;
mod lab;
mod lab_check;
mod cov1p;

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
/// Live bytes the process allocator serves from the system heap (outside the game arena); diagnostics for `play`'s mem lines.
/// The counter lives on the system heap, not in a static: the statics are snapshot state (melee_engine::snapshot), and a
/// rollback load would rewind a static counter while every thread's system-heap blocks stay as they are (and a thread
/// allocating during a load, e.g. slippi-netplay's socket readers, left it differing from the snapshot: MELEE_SNAP_CHECK
/// failed on it). The static only holds the cell's address, set once in `main` before any snapshot base exists.
static SYS_LIVE_CELL: std::sync::atomic::AtomicPtr<std::sync::atomic::AtomicIsize> = std::sync::atomic::AtomicPtr::new(core::ptr::null_mut());
fn sys_live() -> &'static std::sync::atomic::AtomicIsize {
    use std::sync::atomic::{AtomicIsize, Ordering};
    let p = SYS_LIVE_CELL.load(Ordering::Acquire);
    if !p.is_null() {
        return unsafe { &*p };
    }
    // System directly: this runs inside the allocator
    let l = std::alloc::Layout::new::<AtomicIsize>();
    let n = unsafe { std::alloc::GlobalAlloc::alloc(&std::alloc::System, l) } as *mut AtomicIsize;
    assert!(!n.is_null(), "system heap counter");
    unsafe { n.write(AtomicIsize::new(0)) };
    match SYS_LIVE_CELL.compare_exchange(core::ptr::null_mut(), n, Ordering::AcqRel, Ordering::Acquire) {
        Ok(_) => unsafe { &*n },
        Err(won) => {
            unsafe { std::alloc::GlobalAlloc::dealloc(&std::alloc::System, n.cast(), l) };
            unsafe { &*won }
        }
    }
}
/// The system heap's live bytes (see `sys_live`).
pub fn sys_live_bytes() -> isize {
    sys_live().load(std::sync::atomic::Ordering::Relaxed)
}
#[inline]
fn sys_add(n: isize) {
    sys_live().fetch_add(n, std::sync::atomic::Ordering::Relaxed);
}
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
/// `MELEE_BIG_ALLOC=<bytes>`: print a backtrace for each game-arena allocation at least that big (diagnostics).
static BIG_MIN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(usize::MAX);
static IN_TRACE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// `MELEE_ALLOC_SAMPLE=<n>`: also backtrace every n-th game-arena allocation (who allocates per tick).
static SAMPLE_EVERY: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static SAMPLE_N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
fn big_alloc_trace(n: usize) {
    use std::sync::atomic::Ordering::Relaxed;
    let every = SAMPLE_EVERY.load(Relaxed);
    let sampled = every != 0 && SAMPLE_N.fetch_add(1, Relaxed) % every == every - 1;
    if (n < BIG_MIN.load(Relaxed) && !sampled) || IN_TRACE.swap(true, Relaxed) {
        return;
    }
    eprintln!("big alloc {n} bytes:
{}", std::backtrace::Backtrace::force_capture());
    IN_TRACE.store(false, Relaxed);
}
unsafe impl std::alloc::GlobalAlloc for DebugAlloc {
    unsafe fn alloc(&self, l: std::alloc::Layout) -> *mut u8 {
        if melee_engine::arena::active() {
            big_alloc_trace(l.size());
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
        sys_add(l.size() as isize);
        std::alloc::System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: std::alloc::Layout) {
        if melee_engine::arena::contains(p) {
            return melee_engine::arena::free(p, l);
        }
        if rollback_pinned() {
            return; // a snapshot may still point at this system block (see `rollback_pinned`)
        }
        if LEAK_ALLOC.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        #[cfg(windows)]
        if l.size() >= guard_min() && GUARD_ALLOC.load(std::sync::atomic::Ordering::Relaxed) {
            return guard::free(p);
        }
        sys_add(-(l.size() as isize));
        std::alloc::System.dealloc(p, l)
    }
    unsafe fn alloc_zeroed(&self, l: std::alloc::Layout) -> *mut u8 {
        if melee_engine::arena::active() {
            let p = melee_engine::arena::alloc_zeroed(l);
            if !p.is_null() {
                return p;
            }
        }
        #[cfg(windows)]
        if l.size() >= guard_min() && GUARD_ALLOC.load(std::sync::atomic::Ordering::Relaxed) {
            return guard::alloc(l.size(), l.align()); // VirtualAlloc pages are zeroed
        }
        sys_add(l.size() as isize);
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
        if rollback_pinned() || LEAK_ALLOC.load(std::sync::atomic::Ordering::Relaxed) || guarded(l.size()) || guarded(n) {
            let q = self.alloc(std::alloc::Layout::from_size_align_unchecked(n, l.align()));
            if !q.is_null() {
                core::ptr::copy_nonoverlapping(p, q, l.size().min(n));
                self.dealloc(p, l);
            }
            q
        } else {
            let q = std::alloc::System.realloc(p, l, n);
            if !q.is_null() {
                sys_add(n as isize - l.size() as isize);
            }
            q
        }
    }
}
/// A rollback run (snapshot base live) inside the game scope: a system-heap block the game frees or grows is kept
/// (leaked) instead. The game's statics and arena are rolled back by a load, and an older state may still hold the
/// block's address (a Vec the engine created outside the scope, e.g. the GX shim's buffers, then grew inside it);
/// freeing it would leave that state pointing at freed memory. Each such block moves to the arena on its first growth,
/// so the leak is bounded.
#[inline]
fn rollback_pinned() -> bool {
    melee_engine::arena::active() && melee_engine::cow::active()
}
/// [`DebugAlloc`] with the live-heap sampler's hooks (`heapprof`, off unless MELEE_HEAP_SAMPLE is set).
struct Hooked;
unsafe impl std::alloc::GlobalAlloc for Hooked {
    #[inline]
    unsafe fn alloc(&self, l: std::alloc::Layout) -> *mut u8 {
        let p = DebugAlloc.alloc(l);
        if heapprof::enabled() {
            heapprof::on_alloc(p, l.size());
        }
        p
    }
    #[inline]
    unsafe fn dealloc(&self, p: *mut u8, l: std::alloc::Layout) {
        if heapprof::enabled() {
            heapprof::on_free(p);
        }
        DebugAlloc.dealloc(p, l)
    }
    #[inline]
    unsafe fn alloc_zeroed(&self, l: std::alloc::Layout) -> *mut u8 {
        let p = DebugAlloc.alloc_zeroed(l);
        if heapprof::enabled() {
            heapprof::on_alloc(p, l.size());
        }
        p
    }
    #[inline]
    unsafe fn realloc(&self, p: *mut u8, l: std::alloc::Layout, n: usize) -> *mut u8 {
        let q = DebugAlloc.realloc(p, l, n);
        if heapprof::enabled() && !q.is_null() {
            heapprof::on_free(p);
            heapprof::on_alloc(q, n);
        }
        q
    }
}
#[global_allocator]
static GLOBAL: Hooked = Hooked;

/// `arena::set_misuse_hook`: a double free / write after free of an arena block, with the raw return addresses
/// (symbolise with llvm-symbolizer against the exe). This runs under the arena lock, so it must not allocate or take
/// the std stderr lock / formatting machinery: it builds each line in a stack buffer and hands it to `WriteFile`.
#[cfg(windows)]
fn arena_misuse(what: &str, p: usize, n: usize) {
    extern "system" {
        fn RtlCaptureStackBackTrace(skip: u32, count: u32, frames: *mut *mut core::ffi::c_void, hash: *mut u32) -> u16;
        fn GetModuleHandleW(name: *const u16) -> *mut core::ffi::c_void;
        fn GetStdHandle(n: u32) -> *mut core::ffi::c_void;
        fn WriteFile(h: *mut core::ffi::c_void, b: *const u8, n: u32, w: *mut u32, o: *mut core::ffi::c_void) -> i32;
    }
    struct Line { b: [u8; 256], n: usize }
    impl Line {
        fn s(&mut self, t: &[u8]) -> &mut Self {
            for &c in t { if self.n < self.b.len() { self.b[self.n] = c; self.n += 1; } }
            self
        }
        fn hex(&mut self, v: usize) -> &mut Self {
            let mut d = [0u8; 16];
            for (i, x) in d.iter_mut().enumerate() { *x = b"0123456789ABCDEF"[(v >> ((15 - i) * 4)) & 15]; }
            let z = d.iter().position(|&c| c != b'0').unwrap_or(15);
            self.s(b"0x").s(&d[z..])
        }
        fn dec(&mut self, mut v: usize) -> &mut Self {
            let mut d = [0u8; 20];
            let mut i = 20;
            loop { i -= 1; d[i] = b'0' + (v % 10) as u8; v /= 10; if v == 0 { break; } }
            self.s(&d[i..])
        }
        fn out(&mut self) {
            self.s(b"
");
            unsafe {
                let mut w = 0u32;
                WriteFile(GetStdHandle(-12i32 as u32), self.b.as_ptr(), self.n as u32, &mut w, core::ptr::null_mut());
            }
            self.n = 0;
        }
    }
    let mut f = [core::ptr::null_mut::<core::ffi::c_void>(); 40];
    let k = unsafe { RtlCaptureStackBackTrace(0, 40, f.as_mut_ptr(), core::ptr::null_mut()) } as usize;
    let base = unsafe { GetModuleHandleW(core::ptr::null()) } as usize;
    let mut l = Line { b: [0; 256], n: 0 };
    l.s(b"melee: ARENA MISUSE: ").s(what.as_bytes()).s(b": ").hex(p).s(b" (").dec(n).s(b" bytes), game scope ")
        .s(if melee_engine::arena::active() { b"true" } else { b"false" }).out();
    for a in &f[..k] {
        l.s(b"melee:   at melee.exe+").hex((*a as usize).wrapping_sub(base)).out();
    }
}
fn main() -> ExitCode {
    // The YouGame app's stdio bridge (notes/yougame.md): first, before anything can print to stdout or open the log.
    let yougame = melee_platform::yougame::init();
    sys_live(); // the counter's cell exists before any snapshot base is captured
    #[cfg(windows)]
    melee_engine::arena::set_misuse_hook(arena_misuse);
    if std::env::var_os("MELEE_LEAK_ALLOC").is_some() {
        LEAK_ALLOC.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    if let Some(v) = std::env::var_os("MELEE_GUARD_ALLOC") {
        if let Some(n) = v.to_str().and_then(|v| v.parse::<usize>().ok()).filter(|&n| n > 1) {
            GUARD_MIN.store(n, std::sync::atomic::Ordering::Relaxed);
        }
        GUARD_ALLOC.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    if let Some(n) = std::env::var("MELEE_BIG_ALLOC").ok().and_then(|v| v.parse().ok()) {
        BIG_MIN.store(n, std::sync::atomic::Ordering::Relaxed);
    }
    if let Some(n) = std::env::var("MELEE_ALLOC_SAMPLE").ok().and_then(|v| v.parse().ok()) {
        SAMPLE_EVERY.store(n, std::sync::atomic::Ordering::Relaxed);
    }
    heapprof::init();
    let _prof = prof::from_env();
    // opt-in gameplay codes (melee_core::codes, notes/gameplay-codes.md): set before any engine boots
    match melee_core::codes::init_from_args(&std::env::args().skip(1).collect::<Vec<_>>()) {
        Ok(Some(d)) => eprintln!("gw: gameplay codes: {d}"),
        Ok(None) => {}
        Err(e) => {
            eprintln!("melee: {e}");
            return ExitCode::from(2);
        }
    }
    let mut args: std::vec::Vec<String> = std::env::args().skip(1).collect();
    // The Melee Unlocked launcher's command line (launcher_compat.rs): its environment is set before the log redirect.
    let compat = launcher_compat::prepare(&args);
    crash::install();
    if let Some(notes) = yougame {
        eprintln!("yougame: stdio bridge on ({}){}", if melee_platform::yougame::embedded() { "player box" } else { "own window" },
            if notes.is_empty() { String::new() } else { format!(", {}", notes.join(", ")) });
    }
    match compat {
        None => {}
        Some(Err(refused)) => {
            eprintln!("launcher-compat: not supported by the Rust engine yet: {}", refused.join(", "));
            return ExitCode::from(launcher_compat::EXIT_UNSUPPORTED);
        }
        Some(Ok((play_args, notes))) => {
            for n in notes {
                eprintln!("launcher-compat: {n}");
            }
            eprintln!("launcher-compat: melee {}", play_args.join(" "));
            args = play_args;
        }
    }
    if args.windows(2).any(|w| w[0] == "--engine" && w[1] == "legacy") {
        eprintln!("melee: --engine legacy was removed with the vs_match model (consolidation step 19); the engine is the only game loop");
        return ExitCode::from(2);
    }
    match args.first().map(String::as_str) {
        // The launcher's Settings button: the Rust engine has no panel of its own (it reads the port-settings.ini the
        // Source Port's panel writes), so this exits at once and the launcher does not keep a standby copy.
        Some(launcher_compat::SETTINGS_WINDOW) => {
            eprintln!("launcher-compat: --settings-window: the Rust engine has no settings panel; it reads port-settings.ini");
            ExitCode::SUCCESS
        }
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
        Some("version") => {
            let line = std::env::current_exe().ok().and_then(|e| std::fs::read_to_string(e.with_file_name("version.txt")).ok()).and_then(|t| t.lines().next().map(str::to_string));
            println!("melee (rust) {} {}", melee_core::VERSION, line.unwrap_or_else(|| "development build".into()));
            ExitCode::SUCCESS
        }
        Some("crash-test") => crash::crash_test(args.get(1).map(String::as_str).unwrap_or("panic")),
        Some("golden") => golden(&args),
        Some("geno-tests") => geno_suite::run(&args),
        Some("slpcheck") => slpcheck::slpcheck(&args),
        Some("dolphin-wide") => dolphin_oracle::dolphin_wide(&args),
        Some("diff") => diff::diff(&args),
        Some("synctest") => netmatch::engine_game::synctest(&args),
        Some("lab-check") => lab_check::lab_check(&args),
        Some("netplay") => netplay::netplay(&args),
        Some("slippi-netplay") => slippi_np::slippi_netplay(&args),
        Some("slippi-replay") => slippi_np::slippi_replay(&args),
        Some("view-golden") => engine_view::view_golden(&args),
        Some("play") => play::play(&args),
        Some("replays") => replays::replays(&args),
        Some("cosmetics") => cosmetics::cli(&args[1..]),
        Some("spectate") => {
            // `melee spectate TARGET [play options]` = `melee play --spectate TARGET ...`
            let mut a = vec!["play".to_string(), "--spectate".to_string()];
            a.extend(args.iter().skip(1).cloned());
            play::play(&a)
        }
        Some("dat-dump") => retail::dat_dump(&args),
        Some(x) => {
            eprintln!("melee: unknown command {x:?} (selftest | hashlog | golden | diff | view-golden)");
            ExitCode::from(2)
        }
    }
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
        s => return boot_coverage_scenario(iso, s),
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
        e.set_pad_feed(|tick, port| raw_pad_status(pads::pad_fixed(port, tick as i32), port));
    }
    Ok(e)
}

/// One raw scripted sample as the `PADStatus` the engine's pad layer reads (two controllers: ports 3 and 4 report
/// PAD_ERR_NO_CONTROLLER, which the controller indicator's sound reads).
fn raw_pad_status(r: pads::RawPad, port: usize) -> melee_types::PADStatus {
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
}

/// A coverage scenario (docs/rust-port/golden/coverage/scenarios.tsv, notes/coverage.md): the VS ones are wired (scene parse;
/// idle pads feed nothing, the generated rand / cycle pads come from their Lua source); 1p modes return "not wired".
fn boot_coverage_scenario(iso: &str, name: &str) -> Result<melee_engine::Engine, String> {
    // `MELEE_SCENARIO_TSV=FILE`: another table of the same format (the gameplay-code scenarios, tools/rust-port/codes/)
    let tsv_path = std::env::var("MELEE_SCENARIO_TSV").unwrap_or_else(|_| "docs/rust-port/golden/coverage/scenarios.tsv".into());
    let tsv = std::fs::read_to_string(&tsv_path).map_err(|_| format!("scenario {name:?} not wired"))?;
    let row = tsv.lines().find_map(|l| { let c: Vec<&str> = l.split(char::from(9u8)).collect(); (c.len() == 5 && c[0] == name).then(|| (c[1].to_string(), c[3].to_string(), c[4].to_string())) });
    let Some((kind, pad, scene)) = row else { return Err(format!("scenario {name:?} not wired")) };
    if kind == "1p" {
        // the C launch: the boot scene (no memory card) hands over to the mode, whose OnLoad skips its CSS (cov1p.rs)
        let ra = cov1p::RandAbs::from_file(&format!("tools/rust-port/golden/coverage/pads/{pad}"))
            .ok_or(format!("scenario {name:?} not wired: 1P pad script {pad} (only the randabs pads run natively)"))?;
        if std::env::var_os("MELEE_ENGINE_PRE_DRAWS").is_none() {
            // SAFETY: single-threaded start-up, before the engine reads it.
            unsafe { std::env::set_var("MELEE_ENGINE_PRE_DRAWS", "0") };
        }
        let disc = engine_disc(iso)?;
        let scn = melee_engine::scenario::Scenario::parse(&scene)?;
        unsafe {
            melee_engine::engine::BOOT_GAME_MODE = melee_engine::scenario::boot_mode(&scn) as i32;
            // `skipmemcard=0` in the scene (the C's opt-in): the boot scene runs with the card (MELEE_CARD_PATH, else
            // card/ beside the exe), so the run's saves (1P records) reach it, as the C build's same launch
            melee_engine::engine::SKIP_MEMCARD = melee_engine::scenario::skip_memcard(&scene);
            melee_engine::engine::CARD_ON = melee_engine::engine::SKIP_MEMCARD == 0;
        }
        melee_engine::scenario::install(scn);
        let e = melee_engine::Engine::boot_modes(disc, 64 << 20, melee_platform::PadSystem::headless(), melee_types::GM_BOOT as u8)
            .map_err(|e| format!("engine boot: {e}"))?;
        unsafe { COV1P = Some(ra) };
        return Ok(e);
    }
    if kind != "vs" { return Err(format!("scenario {name:?} not wired: {kind} mode (the engine boots VS matches only)")); }
    // idle pads feed nothing; the generated rand / cycle pads are read from their Lua source (pads::LuaPad)
    let lua = if pad.starts_with("idle_") { None } else {
        Some(pads::LuaPad::load(&format!("tools/rust-port/golden/coverage/pads/{pad}"))
            .map_err(|e| format!("scenario {name:?} not wired: pad script {pad}: {e}"))?)
    };
    if std::env::var_os("MELEE_ENGINE_PRE_DRAWS").is_none() {
        // SAFETY: single-threaded start-up, before the engine reads it.
        unsafe { std::env::set_var("MELEE_ENGINE_PRE_DRAWS", "0") };
    }
    let disc = engine_disc(iso)?;
    let scn = melee_engine::scenario::Scenario::parse(&scene)?;
    let mut e = melee_engine::Engine::boot_vs(disc, 64 << 20, melee_platform::PadSystem::headless(), scn).map_err(|e| format!("engine boot: {e}"))?;
    if let Some(lp) = lua {
        e.set_pad_feed(move |tick, port| raw_pad_status(lp.at(port, tick as i32), port));
    }
    Ok(e)
}

/// The consolidated engine is the only game loop (step 19 removed the `vs_match` model). Kept for the callers that asked "engine or model".
pub(crate) fn use_engine(_args: &[String]) -> bool {
    true
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
    let under = engine_gdmods_overlay(args)?;
    // costume skins over the gdmods (cosmetics.rs: only skeleton-identical skins mount, so no hash can change)
    let iso = iso_path(args).ok();
    let sh = if crate::cfg::flag("cosmetics", true) { iso.as_deref().and_then(cosmetics::boot) } else { None };
    cosmetics::set_booted(sh.clone());
    Ok(match sh {
        Some(sh) => Some(cosmetics::overlay(sh, under)),
        None => under,
    })
}

/// `--iso PATH`, else `GW_ISO_VANILLA` (never printed).
pub(crate) fn iso_path(args: &[String]) -> Result<String, String> {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    get("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok()).ok_or_else(|| "no disc image: --iso or GW_ISO_VANILLA".to_string())
}

fn engine_gdmods_overlay(args: &[String]) -> Result<Option<melee_engine::gcm::Overlay>, String> {
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
    // Geno reads each mounted mod's geno.json (geno_registry.c gn_reg: the active mods in id order)
    melee_engine::geno_host::set_boot_mods(&m.dir, &ids);
    Ok(Some(Box::new(move |path: &str| m.lookup(path).and_then(|p| std::fs::read(p).ok()))))
}

/// Route the fighter code's `Replay_*` platform hooks (gw_replay.c) to `pl`: the recorded inputs of the frame being simulated.
/// The VS mode's and the LAB's own `Replay_Active` (their hook tables are separate from the fighter's): Slippi's Neutral Spawn
/// (gmvs.c `fn_8016DEEC` caller) runs only while a replay plays. Set before the match boots.
pub(crate) fn replay_modes_active(slippi_version: i32) {
    unsafe {
        // known before the match starts: the spawn (gmvs neutral_spawn_set) reads it during boot, before install_replay
        melee_runtime::tu::mex_data::SLIPPI_REPLAY_VERSION = Some(slippi_version);
        melee_modes::tu::gmvs::ext::hook::Replay_Active = Some(|| 1);
        melee_modes::tu::geno_lab_mode::ext::hook::Replay_Active = Some(|| 1);
    }
}

pub(crate) fn install_replay(mut pl: melee_data::player::Player) {
    use melee_fighter::platform::hook as h;
    let (codes, ver, frozen) = (pl.slippi_codes(), pl.slippi_version(), pl.slippi_frozen_stadium());
    unsafe {
        // gw_Slippi_Codes / gw_Slippi_Version / gw_Slippi_FrozenStadium while a replay plays
        static mut CODES: i32 = 0;
        static mut FROZEN: i32 = 0;
        CODES = codes;
        FROZEN = frozen as i32;
        h::Slippi_Codes = Some(|| unsafe { CODES });
        // the fighter TUs' own Slippi_Codes hook (ftLib_800864A8's WhispyBlowDirFix, the DeadUpFall freeze, Nana) and
        // Final Destination's (grlast) are separate tables; manual.rs leaves them at 0, so an online replay never
        // turned those codes on
        melee_fighter::hooks::SLIPPI_CODES = Some(|| unsafe { CODES });
        melee_world::tu::grlast::ext::SLIPPI_CODES = Some(|| unsafe { CODES });
        melee_world::tu::grpstadium::ext::hook::Slippi_FrozenStadium = Some(|| unsafe { FROZEN });
        melee_runtime::tu::mex_data::SLIPPI_REPLAY_VERSION = Some(ver);
        REPLAY = Some(pl);
        let r = || REPLAY.as_mut().unwrap();
        h::Replay_Active = Some(|| 1);
        h::Replay_HasInput = Some(|p, f| unsafe {
            slpcheck::seed_probe(p, f);
            // `slpcheck --cpu-ai`: that port's CPU AI makes its inputs (leader and follower)
            if (0..4).contains(&p) && slpcheck::CPU_AI_PORTS.load(std::sync::atomic::Ordering::Relaxed) & (1 << p) != 0 {
                return 0;
            }
            r_in().has_input(p, f != 0) as i32
        });
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
pub(crate) unsafe fn r_in() -> &'static mut melee_data::player::Player {
    REPLAY.as_mut().unwrap()
}

/// The 1P coverage scenario booted by `boot_coverage_scenario` (its randabs pad), for `golden_engine`.
pub(crate) static mut COV1P: Option<cov1p::RandAbs> = None;

/// A 1P coverage run (cov1p.rs): the mode's scenes from the boot on, one CSV row per live match frame (`frame` = the
/// match frame, as the C's `MELEE_NEUTRAL_HASHLOG`), until `rows` rows or the pad script quits.
fn golden_1p(mut e: melee_engine::Engine, ra: cov1p::RandAbs, rows: usize, out: &str) -> ExitCode {
    if let Err(er) = cov1p::start(ra, rows, out) {
        eprintln!("melee: {er}");
        return ExitCode::FAILURE;
    }
    // a scene's begin (gm_801A4D34_begin -> Script_SceneBegin) runs inside tick_modes' mode/scene change
    let mut guard = 0u64;
    let (written, frames) = loop {
        if let Some(d) = cov1p::done() { break d; }
        if e.tick_modes() {
            cov1p::scene_began();
        }
        guard += 1;
        if guard > 1_000_000 { break (0, 0); }
    };
    eprintln!("melee: 1P coverage: {written} rows, {frames} logic frames");
    ExitCode::SUCCESS
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
            Ok((pl, scn, seed)) => {
                // SAFETY: single-threaded start-up, before the engine reads them.
                unsafe {
                    std::env::set_var("MELEE_TEST_SEED", seed.to_string());
                    if std::env::var_os("MELEE_ENGINE_PRE_DRAWS").is_none() { std::env::set_var("MELEE_ENGINE_PRE_DRAWS", "1"); }
                }
                replay_modes_active(pl.slippi_version());
                let booted = engine_disc(&iso).and_then(|disc| {
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
    if let Some(ra) = unsafe { (*std::ptr::addr_of_mut!(COV1P)).take() } {
        // --calltrace on a 1P run: frames are match frames (cov1p's frame_post opens the next one), as the C trace
        let ct_out = get("--calltrace");
        if let Some((a, b)) = get("--calltrace-frames").and_then(|v| v.split_once('-').map(|(a, b)| (a.to_string(), b.to_string()))) {
            if let (Ok(a), Ok(b)) = (a.parse::<i64>(), b.parse::<i64>()) {
                melee_core::ct::set_range(a, b);
            }
        }
        melee_core::ct::begin_frame(-1);
        let rc = golden_1p(e, ra, frames, &out);
        if let Some(p) = &ct_out {
            match melee_core::ct::write(p) {
                Ok(n) => eprintln!("melee: {n} calls written to {p}"),
                Err(er) => eprintln!("melee: {p}: {er}"),
            }
        }
        return rc;
    }
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
    let mut csv_file = std::fs::File::create(&out).ok();
    let mut csv_written = 0usize;
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
    // the practice aids from settings / MELEE_<KEY> / flags: goldens re-run with them on must not change
    let mut practice = practice::Practice::new(practice::Opts::from_args(args));
    for f in 0..frames {
        melee_core::ct::begin_frame(f as i64);
        if melee_core::ct::ENABLED {
            // MELEE_WATCH_FIGHTER=slot,field,fromframe: report the function entry after which the field's u32 changed
            if let Some(v) = std::env::var("MELEE_WATCH_FIGHTER").ok() {
                let p: Vec<&str> = v.split(',').collect();
                if p.len() == 3 && p[2].parse::<usize>().ok() == Some(f) {
                    let g = unsafe { melee_fighter::player::get_entity(p[0].parse().unwrap_or(0)) };
                    if !g.is_null() {
                        let fp = unsafe { (*g).user_data } as usize;
                        let off = match p[1] {
                            "gr_vel" => core::mem::offset_of!(melee_types::Fighter, gr_vel),
                            "motion_id" => core::mem::offset_of!(melee_types::Fighter, motion_id),
                            "xE4" => core::mem::offset_of!(melee_types::Fighter, xE4_ground_accel_1),
                            "xE8" => core::mem::offset_of!(melee_types::Fighter, xE8_ground_accel_2),
                            "mv24" => core::mem::offset_of!(melee_types::Fighter, mv) + 24,
                            "mv28" => core::mem::offset_of!(melee_types::Fighter, mv) + 28,
                            o if o.starts_with("mv") => core::mem::offset_of!(melee_types::Fighter, mv) + o[2..].parse::<usize>().unwrap_or(0),
                            _ => 0,
                        };
                        if off != 0 {
                            melee_core::ct::WATCH_ADDR.store(fp + off, std::sync::atomic::Ordering::Relaxed);
                            eprintln!("watch: fighter slot {} {} at {:#x}", p[0], p[1], fp + off);
                        }
                    }
                }
            }
        }
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
        practice.observe();
        if audio {
            engine_audio::dsp_frame();
        }
        if std::env::var_os("MELEE_PAD_DBG").is_some() {
            let q = unsafe { &melee_runtime::tu::controller::HSD_PadGameStatus[0] };
            eprintln!("PADDBG tick {f} btn {:X} sx {} sy {} nx {} ny {}", q.button, q.stickX, q.stickY, q.nml_stickX, q.nml_stickY);
        }
        if !shots.is_empty() {
            let mut frame = melee_engine::gxbridge::take_frame();
            if let Some(fr) = frame.as_mut().filter(|_| shots.contains(&f)) {
                practice.decorate(fr);
            }
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
        // (appended through one open handle: rewriting the whole file each frame cost ~half the run on Windows)
        if let Some(f) = csv_file.as_mut() {
            use std::io::Write;
            let _ = f.write_all(&csv.as_bytes()[csv_written..]);
            csv_written = csv.len();
        }
    }
    if !practice.log.is_empty() {
        let ok = practice.log.iter().filter(|l| l.1.success).count();
        eprintln!("practice: {} L-cancel landings, {ok} OK; shake suppressed in {} render passes", practice.log.len(), melee_world::tu::camera::SHAKE_SUPPRESSED.load(std::sync::atomic::Ordering::Relaxed));
    } else if practice.opts.no_screen_shake {
        eprintln!("practice: shake suppressed in {} render passes", melee_world::tu::camera::SHAKE_SUPPRESSED.load(std::sync::atomic::Ordering::Relaxed));
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
    drop(csv_file);
    if let Err(er) = std::fs::write(&out, csv) {
        eprintln!("melee: {out}: {er}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// `melee golden`: the consolidated engine (step 19 removed the `vs_match` model: `--engine legacy` and the snapshot-resume flags are gone).
fn golden(args: &[String]) -> ExitCode {
    for gone in ["--from-snapshot", "--dump-state", "--start", "--seed-from"] {
        if args.iter().any(|a| a == gone) {
            eprintln!("melee: {gone} belonged to the removed vs_match model (the engine runs every golden from frame 0)");
            return ExitCode::from(2);
        }
    }
    golden_engine(args)
}

/// Load a replay for `golden --slp`: the player, the scene converted from its Game Start block (slpcheck::convert), the seed.
/// A replay's Gecko list can set `DbLevel` (`melee_data::slp::db_level_code`); both copies of it (staling and the debug
/// gates) follow, and a replay without the code gets the retail `DbLKind_NoDebugRom`.
pub(crate) fn set_replay_db_level(v: Option<i32>) {
    unsafe {
        melee_fighter::stale::DB_LEVEL = v.unwrap_or(0);
        melee_fighter::platform::DB_LEVEL = v.unwrap_or(melee_fighter::platform::DBLKIND_NO_DEBUG_ROM);
    }
}

fn slp_open(path: &str, gdlp: Option<melee_data::gdlp::Identity>) -> Result<(melee_data::player::Player, melee_engine::scenario::Scenario, u32), String> {
    let data = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    // the replay's own widescreen camera code, armed before the match boots (CObjLoad reads it)
    let parsed = melee_data::slp::parse(&data, false).ok();
    let ws = parsed.as_ref().and_then(|f| melee_data::slp::widescreen_code(&f.gecko));
    melee_core::codes::set_replay_widescreen(ws.map(|w| (w.left, w.right, w.num, w.den)));
    melee_core::codes::set_ps_camera_independent_monitor(parsed.as_ref().is_some_and(|f| melee_data::slp::ps_camera_independent_monitor_code(&f.gecko)));
    melee_core::codes::set_ps_preload_transformations(parsed.as_ref().is_some_and(melee_data::slp::ps_preload_transformations));
    melee_core::codes::set_ps_freeze_at_entry(parsed.as_ref().is_some_and(|f| melee_data::slp::ps_freeze_at_entry_code(&f.gecko)));
    let mut pl = melee_data::player::Player::new(melee_data::player::Env { resync: true, identity: gdlp, ..Default::default() });
    if pl.env.slippi_codes.is_none() && parsed.as_ref().is_some_and(|f| melee_data::slp::online_codeset(&f.gecko)) {
        pl.env.slippi_codes = Some("online".into());
    }
    crate::set_replay_db_level(melee_data::slp::db_level_code(parsed.as_ref().map_or(&[][..], |f| &f.gecko[..])));
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
    let (scn, seed) = if gdlp.is_some() {
        // a .gdlp carries the older scene string; keep its two-fighter path
        let scene = pl.r.scene.clone();
        (melee_engine::scenario::Scenario::parse(&scene)?, pl.r.seed)
    } else {
        let s = melee_data::slp::parse(&data, false)?;
        let c = slpcheck::convert(&s)?;
        for n in &c.notes {
            eprintln!("melee: slp: {n}");
        }
        (c.scn, s.seed)
    };
    Ok((pl, scn, seed))
}
