//! Hang watchdog (`MELEE_HANG_DUMP=secs`, inert unless set): a helper thread watches the frame counter the play
//! loops bump ([`beat`]); when it has not moved for `secs` seconds it writes `crashlogs/hang-<time>.log` with every
//! thread's instruction pointer and the code addresses found on its stack (symbolise with llvm-symbolizer against the
//! exe, as for the crash report), then ends the process with exit code 0x48 (72, "H").
//!
//! A frame that legitimately takes longer (a long load) only needs a larger `secs`; nothing else changes.

use std::sync::atomic::{AtomicU64, Ordering};

static BEATS: AtomicU64 = AtomicU64::new(0);

/// One frame done (called by the play loops each tick).
#[inline]
pub fn beat() {
    let n = BEATS.fetch_add(1, Ordering::Relaxed);
    static POISON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *POISON.get_or_init(|| std::env::var_os("MELEE_RECLAIM_POISON").is_some()) {
        melee_engine::archive::check_poison(n);
    }
}

/// Progress without a play-loop frame (the rollback / synctest engine's ticks, netmatch.rs): moves the counter only.
/// The counter is a static, so a snapshot load can put it back; the watchdog only needs it to change.
#[inline]
pub fn pulse() {
    BEATS.fetch_add(1, Ordering::Relaxed);
}

/// Start the watchdog when `MELEE_HANG_DUMP` is set to a number of seconds (>= 2).
pub fn install() {
    let Some(secs) = std::env::var("MELEE_HANG_DUMP").ok().and_then(|v| v.trim().parse::<u64>().ok()) else { return };
    let secs = secs.max(2);
    #[cfg(windows)]
    {
        let main_tid = unsafe { win::GetCurrentThreadId() };
        let _ = std::thread::Builder::new().name("hang-watchdog".into()).spawn(move || watch(secs, main_tid));
    }
    #[cfg(not(windows))]
    let _ = secs;
}

#[cfg(windows)]
fn watch(secs: u64, main_tid: u32) {
    let mut last = BEATS.load(Ordering::Relaxed);
    let mut since = std::time::Instant::now();
    loop {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let now = BEATS.load(Ordering::Relaxed);
        // no frame yet (boot) also counts: a hang before the first frame is still a hang
        if now != last {
            last = now;
            since = std::time::Instant::now();
            continue;
        }
        if since.elapsed().as_secs() >= secs {
            let text = unsafe { dump(main_tid, now, secs) };
            let _ = std::fs::create_dir_all("crashlogs");
            let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
            let path = format!("crashlogs/hang-{t}-{}.log", std::process::id());
            let _ = std::fs::write(&path, &text);
            eprintln!("melee: HANG: no frame for {secs} s (frame counter {now}); thread stacks written to {path}");
            std::process::exit(0x48);
        }
    }
}

/// One thread's sample: id, rip, and up to 200 code addresses from its stack (offset from rsp, address).
#[cfg(windows)]
struct Sample {
    tid: u32,
    rip: usize,
    n: usize,
    st: [(usize, usize); 200],
}

#[cfg(windows)]
unsafe fn dump(main_tid: u32, beats: u64, secs: u64) -> String {
    use std::fmt::Write;
    let base = win::GetModuleHandleW(core::ptr::null()) as usize;
    let me = win::GetCurrentThreadId();
    let pid = win::GetCurrentProcessId();
    let in_exe = |v: usize| v > base && v < base + 0x400_0000;
    // everything allocated up front: a suspended thread may hold the heap lock, so nothing below allocates until
    // every thread has been resumed
    let mut samples: Vec<Sample> = Vec::with_capacity(256);
    let mut handles: Vec<*mut core::ffi::c_void> = Vec::with_capacity(256);
    let snap = win::CreateToolhelp32Snapshot(4 /* TH32CS_SNAPTHREAD */, 0);
    if snap as isize != -1 {
        let mut te: win::THREADENTRY32 = core::mem::zeroed();
        te.dwSize = core::mem::size_of::<win::THREADENTRY32>() as u32;
        let mut ok = win::Thread32First(snap, &mut te);
        while ok != 0 && samples.len() < samples.capacity() {
            if te.th32OwnerProcessID == pid && te.th32ThreadID != me {
                let h = win::OpenThread(0x0002 | 0x0008 | 0x0040 /* SUSPEND, GET_CONTEXT, QUERY */, 0, te.th32ThreadID);
                if !h.is_null() && win::SuspendThread(h) != u32::MAX {
                    handles.push(h);
                    let mut smp = Sample { tid: te.th32ThreadID, rip: 0, n: 0, st: [(0, 0); 200] };
                    let mut ctx = win::Ctx::new();
                    if win::GetThreadContext(h, ctx.ptr()) != 0 {
                        smp.rip = ctx.u64_at(0xF8) as usize;
                        let rsp = ctx.u64_at(0x98) as usize;
                        let mut mbi: win::MEMORY_BASIC_INFORMATION = core::mem::zeroed();
                        if win::VirtualQuery(rsp as *const _, &mut mbi, core::mem::size_of::<win::MEMORY_BASIC_INFORMATION>()) != 0 {
                            let end = mbi.BaseAddress as usize + mbi.RegionSize;
                            let mut p = rsp;
                            while p + 8 <= end && smp.n < 200 {
                                let v = *(p as *const usize);
                                if in_exe(v) {
                                    smp.st[smp.n] = (p - rsp, v - base);
                                    smp.n += 1;
                                }
                                p += 8;
                            }
                        }
                    }
                    samples.push(smp);
                } else if !h.is_null() {
                    win::CloseHandle(h);
                }
            }
            ok = win::Thread32Next(snap, &mut te);
        }
        win::CloseHandle(snap);
    }
    for h in handles {
        win::ResumeThread(h);
        win::CloseHandle(h);
    }
    let mut out = String::new();
    let _ = writeln!(out, "==== GD's Melee hang report ====
no frame for {secs} s; frame counter {beats}
main thread id {main_tid}
addresses are melee.exe+offset (symbolise with llvm-symbolizer --relative-address)
");
    for s in &samples {
        let _ = writeln!(out, "---- thread {}{} ----", s.tid, if s.tid == main_tid { " (main)" } else { "" });
        if in_exe(s.rip) {
            let _ = writeln!(out, "rip: melee.exe+0x{:X}", s.rip - base);
        } else {
            let _ = writeln!(out, "rip: 0x{:X} (outside the exe: a system call / wait)", s.rip);
        }
        for (o, a) in &s.st[..s.n] {
            let _ = writeln!(out, "stack[+0x{o:X}]: melee.exe+0x{a:X}");
        }
    }
    out
}

#[cfg(windows)]
#[allow(non_snake_case, non_camel_case_types, clippy::upper_case_acronyms)]
mod win {
    use core::ffi::c_void;
    #[repr(C)]
    pub struct THREADENTRY32 {
        pub dwSize: u32,
        pub cntUsage: u32,
        pub th32ThreadID: u32,
        pub th32OwnerProcessID: u32,
        pub tpBasePri: i32,
        pub tpDeltaPri: i32,
        pub dwFlags: u32,
    }
    #[repr(C)]
    pub struct MEMORY_BASIC_INFORMATION {
        pub BaseAddress: *mut c_void,
        pub AllocationBase: *mut c_void,
        pub AllocationProtect: u32,
        pub PartitionId: u16,
        pub RegionSize: usize,
        pub State: u32,
        pub Protect: u32,
        pub Type: u32,
    }
    /// An x64 CONTEXT (1232 bytes, 16-aligned) with ContextFlags = CONTEXT_CONTROL | CONTEXT_INTEGER.
    #[repr(C, align(16))]
    pub struct Ctx([u8; 1232]);
    impl Ctx {
        pub fn new() -> Self {
            let mut c = Ctx([0; 1232]);
            c.0[0x30..0x34].copy_from_slice(&0x0010_0003u32.to_le_bytes());
            c
        }
        pub fn ptr(&mut self) -> *mut c_void {
            self.0.as_mut_ptr().cast()
        }
        pub fn u64_at(&self, o: usize) -> u64 {
            u64::from_le_bytes(self.0[o..o + 8].try_into().unwrap())
        }
    }
    extern "system" {
        pub fn GetCurrentThreadId() -> u32;
        pub fn GetCurrentProcessId() -> u32;
        pub fn GetModuleHandleW(name: *const u16) -> *mut c_void;
        pub fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> *mut c_void;
        pub fn Thread32First(snap: *mut c_void, te: *mut THREADENTRY32) -> i32;
        pub fn Thread32Next(snap: *mut c_void, te: *mut THREADENTRY32) -> i32;
        pub fn OpenThread(access: u32, inherit: i32, tid: u32) -> *mut c_void;
        pub fn SuspendThread(h: *mut c_void) -> u32;
        pub fn ResumeThread(h: *mut c_void) -> u32;
        pub fn GetThreadContext(h: *mut c_void, ctx: *mut c_void) -> i32;
        pub fn VirtualQuery(addr: *const c_void, mbi: *mut MEMORY_BASIC_INFORMATION, len: usize) -> usize;
        pub fn CloseHandle(h: *mut c_void) -> i32;
    }
}

