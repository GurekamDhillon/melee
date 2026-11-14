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
    melee_engine::arena::quarantine_check(n);
    static POISON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *POISON.get_or_init(|| std::env::var_os("MELEE_RECLAIM_POISON").is_some()) {
        melee_engine::archive::check_poison(n);
    }
}

/// Start the watchdog when `MELEE_HANG_DUMP` is set to a number of seconds (>= 2).
pub fn install() {
    // initialise the quarantine knob outside the arena lock (its first read allocates)
    melee_engine::arena::quarantine_check(0);
    unsafe { melee_modes::tu::tag_swap::DIAG_IMAGE_OF = Some(melee_engine::archive::image_of) };
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

// ---- write trap (`MELEE_RECLAIM_TRAP=1` with `MELEE_RECLAIM_POISON=1`): the pages of every poisoned object are made
// read-only; a write fault inside a poisoned object writes the writer's stack to crashlogs/trap-<pid>.log and ends
// the process (exit 0x54); a write fault elsewhere on such a page is let through once (page made writable, one
// single step, page protected again). Test-only: slow, and inert unless both knobs are set.
#[cfg(windows)]
mod trap {
    use core::ffi::c_void;
    const N: usize = 1 << 20;
    static mut RANGES: [(usize, usize); N] = [(0, 0); N];
    static mut NR: usize = 0;
    static mut PENDING: usize = 0;
    extern "system" {
        fn VirtualProtect(a: *mut c_void, n: usize, prot: u32, old: *mut u32) -> i32;
        fn AddVectoredExceptionHandler(first: u32, h: unsafe extern "system" fn(*mut Ptrs) -> i32) -> *mut c_void;
        fn GetModuleHandleW(name: *const u16) -> *mut c_void;
        fn ExitProcess(code: u32) -> !;
    }
    #[repr(C)]
    struct Rec {
        code: u32,
        flags: u32,
        rec: *mut Rec,
        addr: *mut c_void,
        nparams: u32,
        info: [usize; 15],
    }
    #[repr(C)]
    struct Ptrs {
        rec: *mut Rec,
        ctx: *mut u8,
    }
    const PAGE: usize = 4096;
    fn protect(p: usize, ro: bool) {
        let mut old = 0u32;
        unsafe { VirtualProtect((p & !(PAGE - 1)) as *mut c_void, PAGE, if ro { 0x02 } else { 0x04 }, &mut old) };
    }
    pub fn add(p: usize, n: usize) {
        unsafe {
            if NR < N {
                RANGES[NR] = (p, p + n);
                NR += 1;
            }
            let mut a = p & !(PAGE - 1);
            while a < p + n {
                protect(a, true);
                a += PAGE;
            }
        }
    }
    unsafe extern "system" fn handler(e: *mut Ptrs) -> i32 {
        let r = &*(*e).rec;
        let ctx = (*e).ctx;
        // EFlags at 0x44, Rip at 0xF8, Rsp at 0x98 (x64 CONTEXT)
        if r.code == 0x8000_0004 && PENDING != 0 {
            // the single step after a let-through write: protect the page again
            protect(PENDING, true);
            PENDING = 0;
            return -1; // EXCEPTION_CONTINUE_EXECUTION
        }
        if r.code != 0xC000_0005 || r.info[0] != 1 {
            return 0; // not a write access violation: not ours
        }
        let a = r.info[1];
        let mut hit = false;
        for i in 0..NR {
            let (lo, hi) = RANGES[i];
            if a >= lo && a < hi {
                hit = true;
                break;
            }
        }
        let page_ours = (0..NR).any(|i| { let (lo, hi) = RANGES[i]; (lo & !(PAGE - 1)) <= a && a < ((hi + PAGE - 1) & !(PAGE - 1)) });
        if !page_ours {
            return 0;
        }
        if !hit {
            protect(a, false);
            PENDING = a;
            *(ctx.add(0x44) as *mut u32) |= 0x100; // trap flag: one instruction, then the single-step above
            return -1;
        }
        // a write into a poisoned (reclaimed) object: the writer's stack, without allocating
        let base = GetModuleHandleW(core::ptr::null()) as usize;
        let rip = *(ctx.add(0xF8) as *const usize);
        let rsp = *(ctx.add(0x98) as *const usize);
        let mut buf = [0u8; 16384];
        let mut n = 0;
        let mut put = |s: &[u8]| { for &c in s { if n < buf.len() { buf[n] = c; n += 1; } } };
        let hex = |v: usize| { let mut d = [0u8; 18]; d[0] = b'0'; d[1] = b'x'; for i in 0..16 { d[2 + i] = b"0123456789ABCDEF"[(v >> ((15 - i) * 4)) & 15]; } d };
        put(b"==== reclaim write trap ====\nwrite to 0x"); put(&hex(a)[2..]); put(b"\nrip melee.exe+"); put(&hex(rip.wrapping_sub(base))); put(b"\n");
        let mut k = 0;
        for i in 0..4096usize {
            let v = *(rsp as *const usize).add(i);
            if v > base && v < base + 0x400_0000 {
                put(b"stack melee.exe+"); put(&hex(v - base)); put(b"\n");
                k += 1;
                if k == 120 { break; }
            }
        }
        let path = b"crashlogs\trap.log\0";
        extern "system" {
            fn CreateFileA(n: *const u8, a: u32, s: u32, sa: *mut c_void, d: u32, f: u32, t: *mut c_void) -> *mut c_void;
            fn WriteFile(h: *mut c_void, b: *const u8, n: u32, w: *mut u32, o: *mut c_void) -> i32;
            fn CloseHandle(h: *mut c_void) -> i32;
        }
        let h = CreateFileA(path.as_ptr(), 0x4000_0000, 0, core::ptr::null_mut(), 2, 0x80, core::ptr::null_mut());
        let mut w = 0u32;
        WriteFile(h, buf.as_ptr(), n as u32, &mut w, core::ptr::null_mut());
        CloseHandle(h);
        ExitProcess(0x54);
    }
    pub fn install() {
        unsafe {
            AddVectoredExceptionHandler(1, handler);
            melee_engine::archive::POISON_HOOK = Some(add);
        }
    }
}

/// `MELEE_RECLAIM_TRAP=1` (with `MELEE_RECLAIM_POISON=1`): install the write trap.
pub fn install_trap() {
    #[cfg(windows)]
    if std::env::var_os("MELEE_RECLAIM_TRAP").is_some() && std::env::var_os("MELEE_RECLAIM_POISON").is_some() {
        let _ = std::fs::create_dir_all("crashlogs");
        trap::install();
    }
}
