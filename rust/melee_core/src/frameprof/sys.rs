//! Machine load for the report (writer thread only): whole-system CPU use, this process's CPU use, and which other
//! heavy processes run beside the game (other `melee*` builds, `cargo` / `rustc` builds, Dolphin). Only executable
//! names are kept, never paths.

/// One machine sample.
#[derive(Clone, Debug, Default)]
pub struct Sample {
    pub t_us: u64,
    /// Percent of all cores busy since the last sample (None: unknown).
    pub cpu: Option<f32>,
    /// This process, percent of one core.
    pub own: Option<f32>,
    /// Other heavy processes: (name, count), sorted by name.
    pub others: Vec<(String, u32)>,
}

impl Sample {
    pub fn others_text(&self) -> String {
        self.others.iter().map(|(n, c)| if *c > 1 { format!("{n} x{c}") } else { n.clone() }).collect::<Vec<_>>().join(", ")
    }
    pub fn other_count(&self) -> u32 {
        self.others.iter().map(|o| o.1).sum()
    }
}

/// Whether an executable name is one of the processes that compete for the machine.
pub fn heavy(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    let stem = n.strip_suffix(".exe").unwrap_or(&n);
    stem.starts_with("melee") || stem == "cargo" || stem == "rustc" || stem.starts_with("dolphin") || stem == "link" || stem == "lld-link" || stem == "clippy-driver"
}

#[cfg(windows)]
mod imp {
    use core::ffi::c_void;
    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct FileTime(u32, u32);
    impl FileTime {
        fn v(self) -> u64 {
            (self.1 as u64) << 32 | self.0 as u64
        }
    }
    #[repr(C)]
    struct Pe32 {
        size: u32,
        usage: u32,
        pid: u32,
        heap: usize,
        module: u32,
        threads: u32,
        parent: u32,
        pri: i32,
        flags: u32,
        exe: [u16; 260],
    }
    extern "system" {
        fn GetSystemTimes(idle: *mut FileTime, kernel: *mut FileTime, user: *mut FileTime) -> i32;
        fn GetProcessTimes(p: *mut c_void, c: *mut FileTime, e: *mut FileTime, k: *mut FileTime, u: *mut FileTime) -> i32;
        fn GetCurrentProcess() -> *mut c_void;
        fn GetCurrentProcessId() -> u32;
        fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> *mut c_void;
        fn Process32FirstW(h: *mut c_void, e: *mut Pe32) -> i32;
        fn Process32NextW(h: *mut c_void, e: *mut Pe32) -> i32;
        fn CloseHandle(h: *mut c_void) -> i32;
    }

    pub fn sys_times() -> (u64, u64) {
        let (mut i, mut k, mut u) = (FileTime::default(), FileTime::default(), FileTime::default());
        // SAFETY: valid out pointers
        if unsafe { GetSystemTimes(&mut i, &mut k, &mut u) } == 0 {
            return (0, 0);
        }
        (i.v(), k.v() + u.v()) // kernel time includes idle time
    }

    #[derive(Default)]
    pub struct Sampler {
        last: Option<(u64, u64, u64, u64, std::time::Instant)>,
    }

    impl Sampler {
        /// (system CPU %, own CPU % of one core) since the last call.
        pub fn cpu(&mut self) -> (Option<f32>, Option<f32>) {
            let (mut i, mut k, mut u) = (FileTime::default(), FileTime::default(), FileTime::default());
            let (mut c, mut e, mut pk, mut pu) = (FileTime::default(), FileTime::default(), FileTime::default(), FileTime::default());
            let ok = unsafe { GetSystemTimes(&mut i, &mut k, &mut u) } != 0 && unsafe { GetProcessTimes(GetCurrentProcess(), &mut c, &mut e, &mut pk, &mut pu) } != 0;
            if !ok {
                return (None, None);
            }
            let now = (i.v(), k.v() + u.v(), pk.v() + pu.v(), 0u64, std::time::Instant::now());
            let r = match self.last {
                Some((li, lt, lp, _, lt0)) if now.1 > lt => {
                    let total = (now.1 - lt) as f64;
                    let idle = now.0.saturating_sub(li) as f64;
                    let wall = (now.4 - lt0).as_secs_f64() * 1e7; // 100 ns units
                    (Some((100.0 * (1.0 - idle / total)).clamp(0.0, 100.0) as f32), (wall > 0.0).then(|| (100.0 * now.2.saturating_sub(lp) as f64 / wall) as f32))
                }
                _ => (None, None),
            };
            self.last = Some(now);
            r
        }

        /// Other heavy processes by name.
        pub fn others(&mut self) -> Vec<(String, u32)> {
            let mut out: Vec<(String, u32)> = Vec::new();
            unsafe {
                let h = CreateToolhelp32Snapshot(2, 0);
                if h.is_null() || h as isize == -1 {
                    return out;
                }
                let me = GetCurrentProcessId();
                let mut e: Pe32 = core::mem::zeroed();
                e.size = core::mem::size_of::<Pe32>() as u32;
                let mut ok = Process32FirstW(h, &mut e) != 0;
                while ok {
                    if e.pid != me {
                        let n = e.exe.iter().position(|&c| c == 0).unwrap_or(e.exe.len());
                        let name = String::from_utf16_lossy(&e.exe[..n]);
                        if super::heavy(&name) {
                            let pri = match e.pri { ..=4 => "idle", 5..=6 => "below normal", 7..=8 => "normal", 9..=10 => "above normal", 11..=13 => "high", _ => "realtime" };
                            let name = format!("{name} [{pri}]");
                            match out.iter_mut().find(|o| o.0.eq_ignore_ascii_case(&name)) {
                                Some(o) => o.1 += 1,
                                None => out.push((name, 1)),
                            }
                        }
                    }
                    ok = Process32NextW(h, &mut e) != 0;
                }
                CloseHandle(h);
            }
            out.sort();
            out
        }
    }
}

#[cfg(not(windows))]
mod imp {
    #[derive(Default)]
    pub struct Sampler;
    impl Sampler {
        pub fn cpu(&mut self) -> (Option<f32>, Option<f32>) {
            (None, None)
        }
        pub fn others(&mut self) -> Vec<(String, u32)> {
            Vec::new()
        }
    }
}

pub use imp::Sampler;

/// Per-frame clocks for the contention flag (game thread, every present; a few hundred ns): the calling thread's
/// and the process's CPU time (ns, cumulative) and the system's idle / total CPU time (100 ns units, cumulative;
/// 0 / 0 when unknown). Thread and process time are cycle counts on Windows (QueryThreadCycleTime, exact, unlike
/// GetThreadTimes' 15.6 ms ticks), turned into ns with the TSC rate measured since `start` (`tsc_hz`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Clocks {
    pub thread_ns: u64,
    pub proc_ns: u64,
    pub sys_idle: u64,
    pub sys_total: u64,
}

#[cfg(all(windows, target_arch = "x86_64"))]
mod clk {
    use core::ffi::c_void;
    extern "system" {
        fn QueryThreadCycleTime(h: *mut c_void, c: *mut u64) -> i32;
        fn QueryProcessCycleTime(h: *mut c_void, c: *mut u64) -> i32;
    }
    /// The system's (idle, total) CPU time, 100 ns units. ~50 µs a call: only the sampler thread calls it.
    pub fn sys_times() -> (u64, u64) {
        super::imp::sys_times()
    }
    static PROC: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    pub fn tsc() -> u64 {
        // SAFETY: rdtsc has no preconditions on x86_64
        unsafe { core::arch::x86_64::_rdtsc() }
    }
    pub fn read(tsc_hz: f64) -> super::Clocks {
        let (mut t, mut p) = (0u64, 0u64);
        // SAFETY: pseudo handles (-2 current thread, -1 current process) and valid out pointers
        unsafe {
            QueryThreadCycleTime(-2isize as *mut c_void, &mut t);
            // QueryProcessCycleTime walks every thread of the process (tens of µs in the game): only when asked
            if *PROC.get_or_init(|| std::env::var_os("MELEE_PROFILE_PROC_CPU").is_some()) {
                QueryProcessCycleTime(-1isize as *mut c_void, &mut p);
            }
        }
        let ns = |c: u64| if tsc_hz > 0.0 { (c as f64 * 1e9 / tsc_hz) as u64 } else { 0 };
        // the system times come from the sampler thread (frameprof::State::sys)
        super::Clocks { thread_ns: ns(t), proc_ns: ns(p), sys_idle: 0, sys_total: 0 }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod clk {
    #[repr(C)]
    struct Ts {
        s: i64,
        ns: i64,
    }
    extern "C" {
        fn clock_gettime(id: i32, t: *mut Ts) -> i32;
    }
    pub fn tsc() -> u64 {
        0
    }
    fn get(id: i32) -> u64 {
        let mut t = Ts { s: 0, ns: 0 };
        // SAFETY: valid out pointer; 2 = CLOCK_PROCESS_CPUTIME_ID, 3 = CLOCK_THREAD_CPUTIME_ID
        if unsafe { clock_gettime(id, &mut t) } != 0 {
            return 0;
        }
        t.s as u64 * 1_000_000_000 + t.ns as u64
    }
    pub fn sys_times() -> (u64, u64) {
        (0, 0)
    }
    pub fn read(_tsc_hz: f64) -> super::Clocks {
        super::Clocks { thread_ns: get(3), proc_ns: get(2), sys_idle: 0, sys_total: 0 }
    }
}

#[cfg(not(any(all(windows, target_arch = "x86_64"), all(unix, not(target_os = "macos")))))]
mod clk {
    pub fn tsc() -> u64 {
        0
    }
    pub fn sys_times() -> (u64, u64) {
        (0, 0)
    }
    pub fn read(_tsc_hz: f64) -> super::Clocks {
        super::Clocks::default()
    }
}

pub use clk::{read as frame_clocks, sys_times, tsc};
