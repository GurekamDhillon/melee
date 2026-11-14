//! Release crash reporting for the shipped `melee` executable.
//!
//! The C build writes `melee-pc.log` and `crashlogs/crash-<time>.log` in its working directory
//! (pc/platform/gw_log.c); the launcher reads both (`latestCrash`, "Open game log", the support
//! bundle). This module gives the Rust build the same two artifacts without touching game logic:
//!
//! * `MELEE_LOG_FILE=<path>`: stderr is redirected (appended) to that file at startup, so every
//!   `eprintln!` of the bin lands in the log the launcher already knows how to open.
//! * A panic hook and (Windows) an unhandled-exception filter write a compact report to
//!   `crashlogs/crash-YYYYMMDD-HHMMSS.log` in the working directory. The first line and the header
//!   keys match gw_log.c's report, so `latestCrash` accepts it. The report holds the reason, a
//!   backtrace, the last 200 lines of the log file and the `MELEE_*` environment, with the user's
//!   profile directory and the disc path replaced by `<user>` / `<disc>`.
//!
//! `melee crash-test panic|fault|abort` raises each kind on purpose; the release smoke test uses it.
use std::io::Write;
use std::path::{Path, PathBuf};

const REPORT_MAX: usize = 64 * 1024;
const LOG_TAIL_LINES: usize = 200;

static LOG_PATH: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
static ISO_PATH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
static REPORTING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Remember the disc path so reports can redact it (`play`/`golden` call this once they know it).
pub fn note_disc(path: &str) {
    let _ = ISO_PATH.set(path.to_string());
}

/// Redirect stderr to `MELEE_LOG_FILE` (when set) and install the crash handlers.
pub fn install() {
    if let Ok(p) = std::env::var("MELEE_LOG_FILE") {
        if !p.is_empty() {
            let _ = LOG_PATH.set(PathBuf::from(&p));
            redirect_stderr(Path::new(&p));
        }
    }
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let loc = info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column())).unwrap_or_default();
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "(non-string panic payload)".to_string()
        };
        prev(info);
        write_report("panic", &format!("{msg} at {loc}"));
    }));
    install_fault_handler();
}

#[cfg_attr(windows, allow(dead_code))]
fn civil_utc(secs: u64) -> (u16, u16, u16, u16, u16, u16) {
    let days = (secs / 86400) as i64;
    let rem = secs % 86400;
    // Howard Hinnant's civil_from_days
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as u16, m as u16, d as u16, (rem / 3600) as u16, (rem % 3600 / 60) as u16, (rem % 60) as u16)
}

#[cfg(windows)]
fn now() -> (u16, u16, u16, u16, u16, u16) {
    #[repr(C)]
    #[derive(Default)]
    struct SystemTime {
        year: u16,
        month: u16,
        dow: u16,
        day: u16,
        hour: u16,
        minute: u16,
        second: u16,
        ms: u16,
    }
    extern "system" {
        fn GetLocalTime(t: *mut SystemTime);
    }
    let mut t = SystemTime::default();
    unsafe { GetLocalTime(&mut t) };
    (t.year, t.month, t.day, t.hour, t.minute, t.second)
}

#[cfg(not(windows))]
fn now() -> (u16, u16, u16, u16, u16, u16) {
    let s = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    civil_utc(s)
}

fn redact(mut s: String) -> String {
    for var in ["USERPROFILE", "HOME"] {
        if let Ok(p) = std::env::var(var) {
            if p.len() > 3 {
                s = s.replace(&p, "<user>").replace(&p.replace('\\', "/"), "<user>");
            }
        }
    }
    for var in ["USERNAME", "USER"] {
        if let Ok(p) = std::env::var(var) {
            if p.len() > 2 {
                s = s.replace(&p, "<user>");
            }
        }
    }
    if let Some(iso) = ISO_PATH.get() {
        if iso.len() > 3 {
            s = s.replace(iso.as_str(), "<disc>").replace(&iso.replace('\\', "/"), "<disc>");
        }
    }
    s
}

fn os_line() -> String {
    format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
}

fn log_tail() -> Vec<String> {
    let Some(p) = LOG_PATH.get() else { return Vec::new() };
    let Ok(mut f) = std::fs::File::open(p) else { return Vec::new() };
    use std::io::{Read, Seek, SeekFrom};
    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
    let take = len.min(48 * 1024);
    let _ = f.seek(SeekFrom::Start(len - take));
    let mut buf = Vec::new();
    let _ = f.read_to_end(&mut buf);
    let text = String::from_utf8_lossy(&buf).into_owned();
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    let from = lines.len().saturating_sub(LOG_TAIL_LINES);
    lines[from..].to_vec()
}

/// Build the report text. `fault` is the exception line(s) for a native fault, empty for a panic.
fn build_report(kind: &str, reason: &str, fault: &str) -> String {
    let (y, mo, d, h, mi, s) = now();
    let exe = std::env::current_exe().ok();
    let version = exe
        .as_ref()
        .and_then(|e| std::fs::read_to_string(e.with_file_name("version.txt")).ok())
        .and_then(|t| t.lines().next().map(str::to_string))
        .unwrap_or_default();
    let build_id = exe
        .as_ref()
        .and_then(|e| std::fs::metadata(e).ok())
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as u32)
        .unwrap_or(0);
    let mut r = String::new();
    r.push_str("==== GD's Melee crash report ====\n");
    r.push_str(&format!("time:            {y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02}\n"));
    r.push_str(&format!("exit path:       {kind}\n"));
    r.push_str(&format!("reason:          {reason}\n"));
    r.push_str("during shutdown: no\n");
    r.push_str("engine:          rust (melee)\n");
    r.push_str(&format!("version:         {}\n", if version.is_empty() { "development build (no version.txt)" } else { &version }));
    r.push_str(&format!("build id:        {build_id:08X} (melee executable file time)\n"));
    r.push_str(&format!("disc:            {}\n", if ISO_PATH.get().is_some() { "<disc>" } else { "no disc yet" }));
    let env: Vec<String> = std::env::vars()
        .filter(|(k, _)| k.starts_with("MELEE_"))
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    r.push_str(&format!("environment:     {}\n", if env.is_empty() { "(no MELEE_* variables)".to_string() } else { env.join(" ") }));
    r.push_str(&format!("os:              {}\n", os_line()));
    r.push_str("\n---- fault ----\n");
    r.push_str(&format!("{kind}: {reason}\n"));
    if !fault.is_empty() {
        r.push_str(fault);
    }
    r.push_str("\n---- backtrace ----\n");
    r.push_str(&std::backtrace::Backtrace::force_capture().to_string());
    let tail = log_tail();
    r.push_str(&format!("\n---- last {} log lines before the fault ----\n", tail.len()));
    for l in &tail {
        r.push_str(l);
        r.push('\n');
    }
    r.push_str("---- end ----\n");
    let mut r = redact(r);
    if r.len() > REPORT_MAX {
        let mut cut = REPORT_MAX - 64;
        while !r.is_char_boundary(cut) {
            cut -= 1;
        }
        r.truncate(cut);
        r.push_str("\n---- truncated ----\n");
    }
    r
}

/// Write `crashlogs/crash-<time>.log` (once per process; a second crash while reporting is ignored).
fn write_report(kind: &str, reason: &str) {
    write_report_with(kind, reason, "");
}

fn write_report_with(kind: &str, reason: &str, fault: &str) {
    if REPORTING.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    if melee_platform::yougame::active() {
        // the YouGame app's "stopped" notice reads the last stderr line
        eprintln!("{}", melee_platform::yougame::crash_line(&format!("{kind}: {reason}")));
    }
    let text = build_report(kind, reason, fault);
    let (y, mo, d, h, mi, s) = now();
    let dir = Path::new("crashlogs");
    let _ = std::fs::create_dir_all(dir);
    let path = dir.join(format!("crash-{y:04}{mo:02}{d:02}-{h:02}{mi:02}{s:02}-{}.log", std::process::id()));
    // MELEE_CRASH_COPY (launcher_compat.rs): the Melee Unlocked launcher reads `melee_port_crash.txt` in the game folder
    // when it is newer than the launch, first line "CRASH: ...".
    if let Ok(copy) = std::env::var("MELEE_CRASH_COPY") {
        if !copy.is_empty() {
            let _ = std::fs::write(&copy, format!("CRASH: {kind}: {reason}, engine rust (melee)
{text}"));
        }
    }
    match std::fs::File::create(&path).and_then(|mut f| f.write_all(text.as_bytes())) {
        Ok(()) => eprintln!("melee: crash report written to {}", path.display()),
        Err(e) => eprintln!("melee: could not write the crash report {}: {e}", path.display()),
    }
}

#[cfg(windows)]
fn raw_frames() -> String {
    extern "system" {
        fn RtlCaptureStackBackTrace(skip: u32, count: u32, frames: *mut *mut core::ffi::c_void, hash: *mut u32) -> u16;
        fn GetModuleHandleW(name: *const u16) -> *mut core::ffi::c_void;
    }
    let mut f = [core::ptr::null_mut::<core::ffi::c_void>(); 48];
    let n = unsafe { RtlCaptureStackBackTrace(0, 48, f.as_mut_ptr(), core::ptr::null_mut()) } as usize;
    let base = unsafe { GetModuleHandleW(core::ptr::null()) } as usize;
    let mut out = String::new();
    for p in &f[..n] {
        let a = *p as usize;
        if a >= base && a < base + (256 << 20) {
            out.push_str(&format!("melee+0x{:X}
", a - base));
        } else {
            out.push_str(&format!("0x{a:X}
"));
        }
    }
    out
}

#[cfg(not(windows))]
fn raw_frames() -> String {
    String::new()
}

// ---- stderr -> file -----------------------------------------------------------------------------

#[cfg(windows)]
fn redirect_stderr(path: &Path) {
    use std::os::windows::io::IntoRawHandle;
    extern "system" {
        fn SetStdHandle(which: u32, h: *mut core::ffi::c_void) -> i32;
    }
    const STD_ERROR_HANDLE: u32 = -12i32 as u32;
    match std::fs::OpenOptions::new().create(true).append(true).open(path) {
        Ok(f) => {
            let h = f.into_raw_handle();
            unsafe { SetStdHandle(STD_ERROR_HANDLE, h as *mut _) };
        }
        Err(e) => eprintln!("melee: MELEE_LOG_FILE {}: {e}", path.display()),
    }
}

#[cfg(unix)]
fn redirect_stderr(path: &Path) {
    use std::os::unix::io::IntoRawFd;
    extern "C" {
        fn dup2(old: i32, new: i32) -> i32;
    }
    match std::fs::OpenOptions::new().create(true).append(true).open(path) {
        Ok(f) => {
            let fd = f.into_raw_fd();
            unsafe { dup2(fd, 2) };
        }
        Err(e) => eprintln!("melee: MELEE_LOG_FILE {}: {e}", path.display()),
    }
}

// ---- native faults ------------------------------------------------------------------------------

#[cfg(windows)]
fn install_fault_handler() {
    #[repr(C)]
    struct ExceptionRecord {
        code: u32,
        flags: u32,
        record: *mut ExceptionRecord,
        address: *mut core::ffi::c_void,
        nparams: u32,
        info: [usize; 15],
    }
    #[repr(C)]
    struct ExceptionPointers {
        record: *mut ExceptionRecord,
        context: *mut core::ffi::c_void,
    }
    extern "system" {
        fn SetUnhandledExceptionFilter(f: Option<unsafe extern "system" fn(*mut ExceptionPointers) -> i32>) -> *mut core::ffi::c_void;
        fn GetModuleHandleW(name: *const u16) -> *mut core::ffi::c_void;
    }
    unsafe extern "system" fn filter(p: *mut ExceptionPointers) -> i32 {
        // the report allocates (format!, the file). A fault inside the game arena's allocator leaves its spin lock
        // held, and the report's first allocation then spun on it forever: the crash looked like a hang at a load.
        // From here this thread allocates from the system heap only.
        melee_engine::arena::mark_foreign_thread();
        let rec = &*(*p).record;
        let base = GetModuleHandleW(core::ptr::null()) as usize;
        let addr = rec.address as usize;
        let what = match rec.code {
            0xC0000005 => {
                let op = match rec.info[0] {
                    0 => "read of",
                    1 => "write to",
                    8 => "execute at",
                    _ => "access to",
                };
                format!("access violation ({op} 0x{:X})", rec.info[1])
            }
            0xC00000FD => "stack overflow".to_string(),
            0xC0000094 => "integer divide by zero".to_string(),
            0xC000001D => "illegal instruction".to_string(),
            0xC0000409 => "fast-fail (abort / stack buffer overrun)".to_string(),
            c => format!("exception 0x{c:08X}"),
        };
        let mut fault = format!(
            "exception code:  0x{:08X}\nexception addr:  0x{:X} (melee.exe+0x{:X})\n",
            rec.code,
            addr,
            addr.wrapping_sub(base)
        );
        // A call through a null pointer faults at rip 0 and the unwinder cannot walk from there, so the backtrace
        // below is empty. List the code addresses on the top of the faulting stack instead (x64 CONTEXT.Rsp at
        // 0x98): the first is the return address into the caller of the null pointer.
        #[cfg(target_arch = "x86_64")]
        if !(*p).context.is_null() {
            let rsp = *((*p).context as *const u8).add(0x98).cast::<usize>();
            let mut n = 0;
            // MELEE_CRASH_STACK=N: list up to N return addresses (default 24), scanning deeper, to reach the caller
            // of a deep recursion (a model load)
            let want = std::env::var("MELEE_CRASH_STACK").ok().and_then(|v| v.parse::<usize>().ok()).unwrap_or(24).clamp(1, 1000);
            // only as far as the stack is readable: a fault on a young thread (a helper a few frames deep) has less
            // than this above its stack pointer, and reading past the stack's top faulted inside this filter, which
            // ended the process with no report at all
            let mut words = (512usize).max(want * 16);
            {
                extern "system" {
                    fn VirtualQuery(addr: *const core::ffi::c_void, info: *mut [usize; 6], len: usize) -> usize;
                }
                let mut info = [0usize; 6];
                let top = if VirtualQuery(rsp as *const _, &mut info, 48) != 0 && info[4] as u32 == 0x1000 { info[0] + info[3] } else { rsp };
                words = words.min(top.saturating_sub(rsp) / 8);
            }
            for i in 0..words {
                let v = *(rsp as *const usize).add(i);
                if v > base && v < base + 0x400_0000 {
                    fault.push_str(&format!("stack[{i:3}]:      melee.exe+0x{:X}\n", v - base));
                    n += 1;
                    if n == want {
                        break;
                    }
                }
            }
        }
        write_report_with("exception", &what, &fault);
        1 // EXCEPTION_EXECUTE_HANDLER: end the process after the report
    }
    unsafe { SetUnhandledExceptionFilter(Some(filter)) };
    // MELEE_CRASH_VEH=1 (diagnostics): a first-chance line for every access violation / stack overflow, on whichever
    // thread it happens, written straight to stderr without allocating: the fault address, the thread and the code
    // addresses on the top of its stack. For faults that end the process before the unhandled filter can report.
    if std::env::var_os("MELEE_CRASH_VEH").is_some() {
        extern "system" {
            fn AddVectoredExceptionHandler(first: u32, h: Option<unsafe extern "system" fn(*mut ExceptionPointers) -> i32>) -> *mut core::ffi::c_void;
            fn GetCurrentThreadId() -> u32;
            fn GetStdHandle(n: u32) -> *mut core::ffi::c_void;
            fn WriteFile(h: *mut core::ffi::c_void, buf: *const u8, len: u32, written: *mut u32, ov: *mut core::ffi::c_void) -> i32;
            fn VirtualQuery(addr: *const core::ffi::c_void, info: *mut [usize; 6], len: usize) -> usize;
        }
        struct Out([u8; 4096], usize);
        impl Out {
            fn s(&mut self, t: &str) {
                for &b in t.as_bytes() {
                    if self.1 < self.0.len() {
                        self.0[self.1] = b;
                        self.1 += 1;
                    }
                }
            }
            fn x(&mut self, v: usize) {
                self.s("0x");
                let mut started = false;
                for i in (0..16).rev() {
                    let d = (v >> (i * 4)) & 15;
                    if d != 0 || started || i == 0 {
                        started = true;
                        let c = [b"0123456789ABCDEF"[d]];
                        self.s(core::str::from_utf8(&c).unwrap_or("?"));
                    }
                }
            }
        }
        unsafe extern "system" fn veh(p: *mut ExceptionPointers) -> i32 {
            let rec = &*(*p).record;
            if rec.code != 0xC0000005 && rec.code != 0xC00000FD {
                return 0;
            }
            let base = GetModuleHandleW(core::ptr::null()) as usize;
            let mut o = Out([0; 4096], 0);
            o.s("melee: first-chance fault code ");
            o.x(rec.code as usize);
            o.s(" at ");
            o.x(rec.address as usize);
            o.s(" (melee.exe+");
            o.x((rec.address as usize).wrapping_sub(base));
            o.s(") access ");
            o.x(rec.info[0]);
            o.s(" of ");
            o.x(rec.info[1]);
            o.s(" thread ");
            o.x(GetCurrentThreadId() as usize);
            o.s("\n");
            #[cfg(target_arch = "x86_64")]
            if !(*p).context.is_null() {
                let rsp = *((*p).context as *const u8).add(0x98).cast::<usize>();
                o.s("  rsp ");
                o.x(rsp);
                // how much of the stack is readable from rsp up
                let mut info = [0usize; 6];
                let mut top = rsp;
                if VirtualQuery(rsp as *const _, &mut info, 48) != 0 && info[4] as u32 == 0x1000 {
                    top = info[0] + info[3];
                }
                o.s(" readable to ");
                o.x(top);
                o.s("\n");
                let mut n = 0;
                let mut a = rsp & !7;
                while a + 8 <= top && n < 48 {
                    let v = *(a as *const usize);
                    if v > base && v < base + 0x400_0000 {
                        o.s("  stack melee.exe+");
                        o.x(v - base);
                        o.s("\n");
                        n += 1;
                    }
                    a += 8;
                }
            }
            let mut w = 0u32;
            WriteFile(GetStdHandle(-12i32 as u32), o.0.as_ptr(), o.1 as u32, &mut w, core::ptr::null_mut());
            0 // EXCEPTION_CONTINUE_SEARCH
        }
        unsafe { AddVectoredExceptionHandler(1, Some(veh)) };
    }
}

#[cfg(not(windows))]
fn install_fault_handler() {
    // SIGSEGV/SIGBUS/SIGILL/SIGFPE: report from the handler on an alternate stack, then re-raise.
    extern "C" {
        fn signal(sig: i32, handler: usize) -> usize;
        fn raise(sig: i32) -> i32;
        fn sigaltstack(new: *const StackT, old: *mut StackT) -> i32;
    }
    #[repr(C)]
    struct StackT {
        sp: *mut core::ffi::c_void,
        flags: i32,
        size: usize,
    }
    extern "C" fn handler(sig: i32) {
        let name = match sig {
            11 => "SIGSEGV (invalid memory access)",
            7 => "SIGBUS",
            4 => "SIGILL (illegal instruction)",
            8 => "SIGFPE",
            6 => "SIGABRT",
            _ => "signal",
        };
        write_report_with("exception", name, &format!("signal: {sig}\n"));
        unsafe {
            signal(sig, 0); // SIG_DFL
            raise(sig);
        }
    }
    unsafe {
        let stack = Box::leak(vec![0u8; 256 * 1024].into_boxed_slice());
        let ss = StackT { sp: stack.as_mut_ptr() as *mut _, flags: 0, size: stack.len() };
        sigaltstack(&ss, core::ptr::null_mut());
        for sig in [11, 7, 4, 8, 6] {
            // `signal` without SA_ONSTACK would run on the faulting stack, which is useless for an
            // overflow; glibc's signal() does not set it, so this covers ordinary faults only.
            signal(sig, handler as usize);
        }
    }
}

/// `melee crash-test panic|fault|abort`: raise a crash on purpose (release smoke test).
pub fn crash_test(kind: &str) -> ! {
    eprintln!("melee: crash-test {kind}");
    match kind {
        "panic" => panic!("crash-test: deliberate panic"),
        "fault" => unsafe {
            let p = core::hint::black_box(8usize) as *const u32;
            let v = core::ptr::read_volatile(p);
            eprintln!("unreachable {v}");
            std::process::exit(0)
        },
        _ => std::process::abort(),
    }
}
