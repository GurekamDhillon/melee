//! Frame profiler (`MELEE_PROFILE=1`, settings key `profile`): what each presented frame and each 60 Hz tick cost,
//! which frames were hitches, and what the game was doing at that moment. Writes `profile-YYYYMMDD-HHMMSS.txt` (plain
//! words first, tables after), `.csv` (one row per presented frame) and `.jsonl` (session, hitches, events, machine
//! samples) into a `profiles` folder; a writer thread rewrites them every few seconds, so a crash or Alt+F4 still
//! leaves a report.
//!
//! Off by default and inert when off: every instrumentation point is one relaxed load of [`ON`] and a branch.
//!
//! Simulation safety (net.md's statics rule): the statics here only hold a flag and a pointer, both set once at
//! start-up before any snapshot base exists, so a rollback load writes back the same values. Everything that changes
//! lives behind the pointer on the system heap, and the game thread never allocates here: events and frames are
//! fixed-size records pushed into vectors whose capacity was reserved by the writer thread (a full vector drops the
//! record and counts it). The recorder only reads clocks and game state; it never writes game memory.
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicU64, AtomicUsize, Ordering::Relaxed};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Recording on (set once by [`start`]).
pub static ON: AtomicBool = AtomicBool::new(false);
static STATE: AtomicPtr<State> = AtomicPtr::new(core::ptr::null_mut());

/// True while recording. Instrumentation points test this first.
#[inline(always)]
pub fn on() -> bool {
    ON.load(Relaxed)
}

/// Per-frame accumulators: time in ns, sizes in bytes, the rest counts. Reset when a frame is recorded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum C {
    /// Ticks run since the last present, their time, the longest one.
    Ticks,
    TickNs,
    TickMaxNs,
    /// Presenter: fresh-frame prepare (geometry, pipelines, textures), draw + interpolation, interpolation evaluate.
    PrepareNs,
    DrawNs,
    InterpNs,
    /// Swapchain image acquire (waits for the display / the GPU queue), blit + overlay + present call.
    AcquireNs,
    PresentNs,
    /// gd_ui presentation frame handed to the renderer (UI mods' draw lists).
    UiNs,
    /// Pacing sleep of the main loop (frame cap / VRR cap), deliberate.
    SleepNs,
    /// Disc reads: files, bytes, time.
    Files,
    FileBytes,
    FileNs,
    /// Archive parses and their time.
    Parses,
    ParseNs,
    /// GPU pipelines (shaders) created and the time; textures created + uploaded, bytes, time.
    Pipes,
    PipeNs,
    Textures,
    TexBytes,
    TexNs,
    /// Big allocations (>= 4 MiB) and bytes.
    BigAllocs,
    BigAllocBytes,
    /// Game arena allocations (count).
    ArenaAllocs,
    /// Window events (focus, resize, fullscreen switch).
    WindowEvents,
    /// Inside a tick: the engine's logic step (`RetailPlay::step`, mods' sim hooks included) and the frame capture
    /// handed to the presenter (take / detach / EFB copies); the rest of the tick is harness work (UI mods, audio,
    /// scene bookkeeping).
    EngineNs,
    CaptureNs,
    /// Wasm mod time, all mods (the per-mod split is in the frame's mod slots).
    ModNs,
}
pub const NC: usize = C::ModNs as usize + 1;
/// Mods the per-mod split keeps apart.
pub const MAX_MODS: usize = 8;

/// What an event is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    Scene = 0,
    File,
    Stage,
    Fighter,
    Envoy,
    Window,
    Alloc,
    Wasm,
    Test,
    Note,
    Audio,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Scene => "scene",
            Kind::File => "file",
            Kind::Stage => "stage",
            Kind::Fighter => "fighter",
            Kind::Envoy => "envoy",
            Kind::Window => "window",
            Kind::Alloc => "alloc",
            Kind::Wasm => "wasm",
            Kind::Test => "test",
            Kind::Note => "note",
            Kind::Audio => "audio",
        }
    }
    fn from_u8(v: u8) -> Kind {
        [Kind::Scene, Kind::File, Kind::Stage, Kind::Fighter, Kind::Envoy, Kind::Window, Kind::Alloc, Kind::Wasm, Kind::Test, Kind::Note, Kind::Audio].get(v as usize).copied().unwrap_or(Kind::Note)
    }
}

const TEXT: usize = 56;

/// A fixed-size text built without allocating (event details on the game thread).
#[derive(Clone, Copy)]
pub struct Text {
    b: [u8; TEXT],
    n: u8,
}

impl Default for Text {
    fn default() -> Self {
        Text { b: [0; TEXT], n: 0 }
    }
}

impl Text {
    pub fn new() -> Text {
        Text::default()
    }
    pub fn s(mut self, t: &str) -> Text {
        self.push_bytes(t.as_bytes());
        self
    }
    pub fn b(mut self, t: &[u8]) -> Text {
        self.push_bytes(t);
        self
    }
    pub fn n(mut self, v: i64) -> Text {
        let mut d = [0u8; 20];
        let mut i = d.len();
        let neg = v < 0;
        let mut u = v.unsigned_abs();
        loop {
            i -= 1;
            d[i] = b'0' + (u % 10) as u8;
            u /= 10;
            if u == 0 {
                break;
            }
        }
        if neg {
            self.push_bytes(b"-");
        }
        self.push_bytes(&d[i..]);
        self
    }
    fn push_bytes(&mut self, t: &[u8]) {
        for &c in t {
            if (self.n as usize) < TEXT && c != 0 {
                // printable ASCII only (a file name from the disc's table, never anything else)
                self.b[self.n as usize] = if (0x20..0x7F).contains(&c) && c != b'"' && c != b'\\' { c } else { b'?' };
                self.n += 1;
            }
        }
    }
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.b[..self.n as usize]).unwrap_or("")
    }
}

#[derive(Clone, Copy)]
struct Event {
    t_us: u64,
    tick: u64,
    kind: u8,
    dur_us: u32,
    val: u64,
    text: Text,
}

#[derive(Clone, Copy)]
struct Frame {
    /// End of this present (µs since the session started) and the interval since the previous one.
    t_us: u64,
    interval_us: u32,
    scene: u16,
    tick: u64,
    c: [u64; NC],
    mods: [u64; MAX_MODS],
    interp: bool,
    expected_us: u32,
}

/// Start-up options (see `melee::play`).
#[derive(Clone, Debug)]
pub struct Config {
    /// The folder the reports go to (created).
    pub dir: std::path::PathBuf,
    /// A present is a hitch above `frame_mult` x the display's frame interval; a tick above `tick_mult` x 16.7 ms.
    pub frame_mult: f64,
    pub tick_mult: f64,
    /// The on-screen readout.
    pub overlay: bool,
    /// Test knob: (tick, ms) sleep once in that tick (an induced hitch).
    pub test_sleep: Option<(u64, u64)>,
}

struct ModSlot {
    name: [u8; 24],
    n: u8,
}

struct State {
    t0: Instant,
    cfg: Config,
    c: [AtomicU64; NC],
    mod_ns: [AtomicU64; MAX_MODS],
    mod_names: Mutex<Vec<ModSlot>>,
    mod_count: AtomicUsize,
    /// scene (mode << 8 | kind) and the game's tick counter
    scene: AtomicU32,
    tick: AtomicU64,
    frames: Mutex<Vec<Frame>>,
    events: Mutex<Vec<Event>>,
    dropped: AtomicU64,
    info: Mutex<Vec<(String, String)>>,
    /// expected present interval (µs), from the main loop's pacing
    expected_us: AtomicU64,
    interp: AtomicBool,
    last_present: AtomicU64,
    /// the overlay's ring of the last intervals (µs) and the hitch count / last hitch
    graph: [AtomicU32; GRAPH],
    graph_i: AtomicUsize,
    hitches: AtomicU64,
    last_hitch: AtomicU64,
    stop: AtomicBool,
    writer: Mutex<Option<std::thread::JoinHandle<()>>>,
    test_slept: AtomicBool,
    audio_underruns: AtomicU64,
}

pub const GRAPH: usize = 240;
const FRAME_CAP: usize = 16384;
const EVENT_CAP: usize = 4096;

fn st() -> Option<&'static State> {
    let p = STATE.load(Relaxed);
    if p.is_null() {
        None
    } else {
        Some(unsafe { &*p })
    }
}

/// Microseconds since the session started.
fn now_us(s: &State) -> u64 {
    s.t0.elapsed().as_micros() as u64
}

/// Start recording (once; call before any snapshot base exists and outside the game scope). Returns the report's
/// file stem, or an error when the folder cannot be written.
pub fn start(cfg: Config) -> Result<String, String> {
    if !STATE.load(Relaxed).is_null() {
        return Err("already started".into());
    }
    std::fs::create_dir_all(&cfg.dir).map_err(|e| format!("profiles folder: {e}"))?;
    let stamp = local_stamp();
    let stem = format!("profile-{stamp}");
    // a probe write: a read-only folder fails here, and the caller picks another
    let probe = cfg.dir.join(format!("{stem}.txt"));
    std::fs::write(&probe, "profiling...\n").map_err(|e| format!("profiles folder not writable: {e}"))?;
    let s = Box::new(State {
        t0: Instant::now(),
        cfg,
        c: Default::default(),
        mod_ns: Default::default(),
        mod_names: Mutex::new(Vec::with_capacity(MAX_MODS)),
        mod_count: AtomicUsize::new(0),
        scene: AtomicU32::new(0),
        tick: AtomicU64::new(0),
        frames: Mutex::new(Vec::with_capacity(FRAME_CAP)),
        events: Mutex::new(Vec::with_capacity(EVENT_CAP)),
        dropped: AtomicU64::new(0),
        info: Mutex::new(Vec::new()),
        expected_us: AtomicU64::new(16_667),
        interp: AtomicBool::new(false),
        last_present: AtomicU64::new(0),
        graph: [const { AtomicU32::new(0) }; GRAPH],
        graph_i: AtomicUsize::new(0),
        hitches: AtomicU64::new(0),
        last_hitch: AtomicU64::new(0),
        stop: AtomicBool::new(false),
        writer: Mutex::new(None),
        test_slept: AtomicBool::new(false),
        audio_underruns: AtomicU64::new(0),
    });
    let p = Box::into_raw(s);
    STATE.store(p, Relaxed);
    ON.store(true, Relaxed);
    let s: &'static State = unsafe { &*p };
    let st2 = stem.clone();
    let h = std::thread::Builder::new().name("frame profiler".into()).spawn(move || writer::run(s, st2)).map_err(|e| e.to_string())?;
    *s.writer.lock().unwrap() = Some(h);
    Ok(stem)
}

/// Stop: final drain and report (call once at exit, outside the game scope). Safe to call when never started.
pub fn finish() {
    let Some(s) = st() else { return };
    if s.stop.swap(true, Relaxed) {
        return;
    }
    if let Some(h) = s.writer.lock().unwrap().take() {
        let _ = h.join();
    }
}

/// Session facts for the report header (display settings, GPU, build, mods). Outside the game scope only.
pub fn info(key: &str, value: &str) {
    let Some(s) = st() else { return };
    let mut v = s.info.lock().unwrap();
    if let Some(e) = v.iter_mut().find(|e| e.0 == key) {
        e.1 = value.to_string();
    } else {
        v.push((key.to_string(), value.to_string()));
    }
}

/// Add to a per-frame accumulator.
#[inline]
pub fn add(c: C, v: u64) {
    if let Some(s) = st() {
        s.c[c as usize].fetch_add(v, Relaxed);
    }
}

/// One tick's time (adds to the frame's ticks, time and longest tick).
#[inline]
pub fn tick_done(ns: u64) {
    if let Some(s) = st() {
        s.c[C::Ticks as usize].fetch_add(1, Relaxed);
        s.c[C::TickNs as usize].fetch_add(ns, Relaxed);
        s.c[C::TickMaxNs as usize].fetch_max(ns, Relaxed);
    }
}

/// Time a closure into `c` when recording (the closure runs either way).
#[inline]
pub fn time<R>(c: C, f: impl FnOnce() -> R) -> R {
    if !on() {
        return f();
    }
    let t = Instant::now();
    let r = f();
    add(c, t.elapsed().as_nanos() as u64);
    r
}

/// A mod's slot for [`mod_ns`] (registered by name at load; at most [`MAX_MODS`], the rest share the last).
pub fn mod_slot(name: &str) -> u8 {
    let Some(s) = st() else { return 0 };
    let mut v = s.mod_names.lock().unwrap();
    if let Some(i) = v.iter().position(|m| &m.name[..m.n as usize] == name.as_bytes()) {
        return i as u8;
    }
    if v.len() >= MAX_MODS {
        return (MAX_MODS - 1) as u8;
    }
    let mut m = ModSlot { name: [0; 24], n: 0 };
    for (i, c) in name.bytes().take(24).enumerate() {
        m.name[i] = c;
        m.n = i as u8 + 1;
    }
    v.push(m);
    s.mod_count.store(v.len(), Relaxed);
    (v.len() - 1) as u8
}

/// Time spent in one mod's call.
#[inline]
pub fn mod_ns(slot: u8, ns: u64) {
    if let Some(s) = st() {
        s.mod_ns[(slot as usize).min(MAX_MODS - 1)].fetch_add(ns, Relaxed);
        s.c[C::ModNs as usize].fetch_add(ns, Relaxed);
    }
}

/// The game's scene (major mode, scene kind) and tick, set every tick.
#[inline]
pub fn set_scene(mode: u8, kind: u8, tick: u64) {
    if let Some(s) = st() {
        s.scene.store((mode as u32) << 8 | kind as u32, Relaxed);
        s.tick.store(tick, Relaxed);
    }
}

/// The audio device's underrun count so far (a rise is logged as an event).
pub fn set_audio_underruns(n: u64) {
    if let Some(s) = st() {
        let was = s.audio_underruns.swap(n, Relaxed);
        if n > was {
            event(Kind::Audio, Text::new().s("audio underrun +").n((n - was) as i64).s(" frames"), 0, n - was);
        }
    }
}

/// The present interval the display pace asks for (µs), and whether interpolation is on.
pub fn set_pacing(expected_us: u64, interp: bool) {
    if let Some(s) = st() {
        s.expected_us.store(expected_us.max(1000), Relaxed);
        s.interp.store(interp, Relaxed);
    }
}

/// Record an event (no allocation; dropped when the buffer is full).
pub fn event(kind: Kind, text: Text, dur_us: u32, val: u64) {
    let Some(s) = st() else { return };
    let e = Event { t_us: now_us(s), tick: s.tick.load(Relaxed), kind: kind as u8, dur_us, val, text };
    let mut v = s.events.lock().unwrap();
    if v.len() < v.capacity() {
        v.push(e);
    } else {
        s.dropped.fetch_add(1, Relaxed);
    }
}

thread_local! {
    static WRITER: core::cell::Cell<bool> = const { core::cell::Cell::new(false) };
}

/// An allocation seen by the process allocator (`arena`: the game arena served it). Allocation-free: safe inside the
/// allocator. Allocations of the profiler's own writer thread are not counted.
#[inline]
pub fn alloc(size: usize, arena: bool) {
    if arena {
        add(C::ArenaAllocs, 1);
    }
    if size >= 4 << 20 && !WRITER.with(|w| w.get()) {
        add(C::BigAllocs, 1);
        add(C::BigAllocBytes, size as u64);
        event(Kind::Alloc, Text::new().s("allocation ").n((size >> 10) as i64).s(" KB").s(if arena { " (game arena)" } else { " (system heap)" }), 0, size as u64);
    }
}

/// An event that times the scope it lives in (recorded when dropped). Inert when not recording.
pub struct Span(Option<(Kind, Text, Instant)>);

impl Span {
    #[inline]
    pub fn new(kind: Kind, text: impl FnOnce() -> Text) -> Span {
        Span(if on() { Some((kind, text(), Instant::now())) } else { None })
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        if let Some((k, t, t0)) = self.0.take() {
            let us = t0.elapsed().as_micros() as u64;
            event(k, t.s(" (").n((us / 1000) as i64).s(" ms)"), us.min(u32::MAX as u64) as u32, 0);
        }
    }
}

/// A [`Span`] recorded only when it took at least `min_us` (per-frame phases: nothing is logged for a normal frame).
pub struct SlowSpan(Option<(Kind, &'static str, Instant, u32)>);

impl SlowSpan {
    #[inline]
    pub fn new(kind: Kind, min_us: u32, text: &'static str) -> SlowSpan {
        SlowSpan(if on() { Some((kind, text, Instant::now(), min_us)) } else { None })
    }
}

impl Drop for SlowSpan {
    fn drop(&mut self) {
        if let Some((k, t, t0, min)) = self.0.take() {
            let us = t0.elapsed().as_micros() as u64;
            if us >= min as u64 {
                event(k, Text::new().s(t).s(" (").n((us / 1000) as i64).s(" ms)"), us.min(u32::MAX as u64) as u32, 0);
            }
        }
    }
}

/// An instant event built only when recording.
#[inline]
pub fn note(kind: Kind, text: impl FnOnce() -> Text) {
    if on() {
        event(kind, text(), 0, 0);
    }
}

/// End of a present: record the frame (its interval and the accumulators since the last one). Main thread.
pub fn end_frame() {
    let Some(s) = st() else { return };
    let t = now_us(s);
    let last = s.last_present.swap(t, Relaxed);
    let interval = if last == 0 { 0 } else { (t - last).min(u32::MAX as u64) as u32 };
    let mut f = Frame { t_us: t, interval_us: interval, scene: s.scene.load(Relaxed) as u16, tick: s.tick.load(Relaxed), c: [0; NC], mods: [0; MAX_MODS], interp: s.interp.load(Relaxed), expected_us: s.expected_us.load(Relaxed) as u32 };
    for (i, c) in s.c.iter().enumerate() {
        f.c[i] = c.swap(0, Relaxed);
    }
    for (i, m) in s.mod_ns.iter().enumerate() {
        f.mods[i] = m.swap(0, Relaxed);
    }
    if last != 0 {
        let gi = s.graph_i.fetch_add(1, Relaxed) % GRAPH;
        s.graph[gi].store(interval, Relaxed);
        let exp = s.expected_us.load(Relaxed) as f64;
        if interval as f64 > s.cfg.frame_mult * exp || f.c[C::TickMaxNs as usize] as f64 > s.cfg.tick_mult * 16_667_000.0 {
            s.hitches.fetch_add(1, Relaxed);
            s.last_hitch.store((t << 20) | (interval as u64 / 1000).min(0xFFFFF), Relaxed);
        }
    }
    let mut v = s.frames.lock().unwrap();
    if v.len() < v.capacity() {
        v.push(f);
    } else {
        s.dropped.fetch_add(1, Relaxed);
    }
}

/// Test knob (`MELEE_PROFILE_TEST_SLEEP=tick:ms`): sleep once in that tick, an induced hitch for checking the report.
pub fn test_tick(tick: u64) {
    let Some(s) = st() else { return };
    if let Some((at, ms)) = s.cfg.test_sleep {
        if tick >= at && !s.test_slept.swap(true, Relaxed) {
            event(Kind::Test, Text::new().s("test sleep ").n(ms as i64).s(" ms"), (ms * 1000) as u32, ms);
            std::thread::sleep(Duration::from_millis(ms));
        }
    }
}

/// For the on-screen readout: the last [`GRAPH`] present intervals (µs, oldest first), the expected interval, hitches
/// so far, and the last hitch (seconds ago, ms).
pub struct Readout {
    pub intervals: [u32; GRAPH],
    pub expected_us: u64,
    pub hitches: u64,
    pub last: Option<(f64, u64)>,
}

pub fn overlay_on() -> bool {
    on() && st().is_some_and(|s| s.cfg.overlay)
}

pub fn readout() -> Option<Readout> {
    let s = st()?;
    let mut r = Readout { intervals: [0; GRAPH], expected_us: s.expected_us.load(Relaxed), hitches: s.hitches.load(Relaxed), last: None };
    let i0 = s.graph_i.load(Relaxed);
    for k in 0..GRAPH {
        r.intervals[k] = s.graph[(i0 + k) % GRAPH].load(Relaxed);
    }
    let lh = s.last_hitch.load(Relaxed);
    if lh != 0 {
        let at = lh >> 20;
        r.last = Some((now_us(s).saturating_sub(at) as f64 / 1e6, lh & 0xFFFFF));
    }
    Some(r)
}

/// Local date and time as YYYYMMDD-HHMMSS (Windows: the local clock; elsewhere UTC).
fn local_stamp() -> String {
    #[cfg(windows)]
    {
        #[repr(C)]
        #[derive(Default)]
        struct SystemTime {
            y: u16,
            m: u16,
            dow: u16,
            d: u16,
            h: u16,
            mi: u16,
            s: u16,
            ms: u16,
        }
        extern "system" {
            fn GetLocalTime(t: *mut SystemTime);
        }
        let mut t = SystemTime::default();
        unsafe { GetLocalTime(&mut t) };
        format!("{:04}{:02}{:02}-{:02}{:02}{:02}", t.y, t.m, t.d, t.h, t.mi, t.s)
    }
    #[cfg(not(windows))]
    {
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
        let (days, rem) = (secs / 86400, secs % 86400);
        // civil from days (Howard Hinnant)
        let z = days as i64 + 719468;
        let era = z.div_euclid(146097);
        let doe = z - era * 146097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = yoe + era * 400 + (m <= 2) as i64;
        format!("{y:04}{m:02}{d:02}-{:02}{:02}{:02}", rem / 3600, rem / 60 % 60, rem % 60)
    }
}

mod sys;
mod writer;

#[cfg(test)]
mod tests;
