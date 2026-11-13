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
    maybe_refault_test();
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
    // a native fault already wrote the raw stage of the report (`fault_stage1`) under a name: replace that file
    let path = match raw::path() {
        Some(p) => PathBuf::from(p),
        None => {
            let (y, mo, d, h, mi, s) = now();
            let dir = Path::new("crashlogs");
            let _ = std::fs::create_dir_all(dir);
            dir.join(format!("crash-{y:04}{mo:02}{d:02}-{h:02}{mi:02}{s:02}-{}.log", std::process::id()))
        }
    };
    // MELEE_CRASH_COPY (launcher_compat.rs): the Melee Unlocked launcher reads `melee_port_crash.txt` in the game folder
    // when it is newer than the launch, first line "CRASH: ...".
    if let Ok(copy) = std::env::var("MELEE_CRASH_COPY") {
        if !copy.is_empty() {
            let _ = std::fs::write(&copy, format!("CRASH: {kind}: {reason}, engine rust (melee)\n{text}"));
        }
    }
    // written beside and renamed over, so a fault while writing leaves the raw stage in place
    raw::close();
    let tmp = path.with_extension("log.tmp");
    let res = std::fs::File::create(&tmp)
        .and_then(|mut f| f.write_all(text.as_bytes()))
        .and_then(|_| std::fs::rename(&tmp, &path));
    match res {
        Ok(()) => eprintln!("melee: crash report written to {}", path.display()),
        Err(e) => eprintln!("melee: could not write the crash report {}: {e}", path.display()),
    }
}

// ---- fault path without the game arena ------------------------------------------------------------
//
// A native fault can come from a corrupt game arena (the global allocator's backing store, melee_engine::arena), and
// the report used to allocate through it (formatting, std's backtrace symbolisation): the handler faulted a second
// time and no report was written (Linux soak). On a fault:
// 1. `SYSTEM_ONLY` is set first: from then on the global allocator (`DebugAlloc` in main.rs) serves every allocation
//    of every thread from the system heap and leaks every free, so nothing touches the arena again;
// 2. stage 1 writes the report's header, the fault and the raw return addresses with no allocation at all (static
//    buffers, the OS's own file calls) to `crashlogs/crash-<time>-<pid>.log`;
// 3. stage 2 builds the full symbolised report (system heap) and renames it over the stage-1 file;
// 4. a second fault inside the handler (`FAULT_DEPTH`) appends one line to the stage-1 file and ends the process.

static SYSTEM_ONLY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static FAULT_DEPTH: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
/// `MELEE_CRASH_TEST_REFAULT=1` (crash-test only): fault again inside stage 2.
static REFAULT_TEST: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// `MELEE_CRASH_STACK`, read at install (the handler must not touch the environment before stage 1).
#[cfg_attr(not(windows), allow(dead_code))]
static CRASH_STACK: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(24);
/// Load address of the executable, for exe-relative frame addresses.
static EXE_BASE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// True once a native fault is being reported: the global allocator then stays off the game arena (main.rs).
#[inline]
pub fn system_only() -> bool {
    SYSTEM_ONLY.load(std::sync::atomic::Ordering::Relaxed)
}

/// A fixed buffer `core::fmt` writes into without allocating (output past the end is dropped).
struct FixedBuf<const N: usize> {
    b: [u8; N],
    n: usize,
}
impl<const N: usize> FixedBuf<N> {
    const fn new() -> Self {
        FixedBuf { b: [0; N], n: 0 }
    }
    fn as_str(&self) -> &str {
        // only whole `&str`s are copied in, cut at a char boundary
        unsafe { core::str::from_utf8_unchecked(&self.b[..self.n]) }
    }
}
impl<const N: usize> core::fmt::Write for FixedBuf<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut take = s.len().min(N - self.n);
        while !s.is_char_boundary(take) {
            take -= 1;
        }
        self.b[self.n..self.n + take].copy_from_slice(&s.as_bytes()[..take]);
        self.n += take;
        Ok(())
    }
}

// only the first fault uses these (FAULT_DEPTH), so plain statics are enough; static, not on the stack, because a
// stack overflow is one of the faults reported
static mut WHAT: FixedBuf<192> = FixedBuf::new();
static mut FAULT: FixedBuf<16384> = FixedBuf::new();
static mut STAGE1: FixedBuf<20480> = FixedBuf::new();
static mut FRAMES: [usize; 64] = [0; 64];

/// Append the raw frames as `frame[ i]: melee+0x...` lines.
fn push_frames(out: &mut impl core::fmt::Write, frames: &[usize]) {
    let base = EXE_BASE.load(std::sync::atomic::Ordering::Relaxed);
    let _ = out.write_str("raw frames (return addresses; symbolise against the executable):\n");
    for (i, &a) in frames.iter().enumerate() {
        let _ = if base != 0 && a >= base && a < base + (256 << 20) {
            writeln!(out, "frame[{i:2}]:       melee+0x{:X}", a - base)
        } else {
            writeln!(out, "frame[{i:2}]:       0x{a:X}")
        };
    }
}

/// Stage 1: header, fault and raw frames, written with no allocation. Keeps the file open for `refault`.
unsafe fn fault_stage1(what: &str, fault: &str) {
    use core::fmt::Write as _;
    let (y, mo, d, h, mi, s) = now();
    let r = &mut *core::ptr::addr_of_mut!(STAGE1);
    let _ = r.write_str("==== GD's Melee crash report ====\n");
    let _ = writeln!(r, "time:            {y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02}");
    let _ = r.write_str("exit path:       exception\n");
    let _ = writeln!(r, "reason:          {what}");
    let _ = r.write_str("during shutdown: no\nengine:          rust (melee)\n");
    let _ = writeln!(r, "os:              {} {}", std::env::consts::OS, std::env::consts::ARCH);
    let _ = r.write_str("report stage:    raw (written without allocating; the symbolised report replaces this file when it completes)\n");
    let _ = r.write_str("\n---- fault ----\n");
    let _ = writeln!(r, "exception: {what}");
    let _ = r.write_str(fault);
    let _ = r.write_str("---- end of raw stage ----\n");
    raw::open_write(y, mo, d, h, mi, s, r.as_str().as_bytes());
}

/// A fault inside the handler: note it in the stage-1 file and end the process.
unsafe fn refault(code: i32) -> ! {
    raw::append(b"\n---- second fault inside the crash handler: the symbolised report was abandoned, the raw stage above is the report ----\n");
    raw::close();
    raw::exit_now(code)
}

#[cfg(windows)]
mod raw {
    type Handle = *mut core::ffi::c_void;
    extern "system" {
        fn CreateDirectoryA(path: *const u8, sa: *mut core::ffi::c_void) -> i32;
        fn CreateFileA(path: *const u8, access: u32, share: u32, sa: *mut core::ffi::c_void, disp: u32, flags: u32, tmpl: Handle) -> Handle;
        fn WriteFile(h: Handle, b: *const u8, n: u32, w: *mut u32, o: *mut core::ffi::c_void) -> i32;
        fn FlushFileBuffers(h: Handle) -> i32;
        fn CloseHandle(h: Handle) -> i32;
        fn GetCurrentProcessId() -> u32;
        fn GetCurrentProcess() -> Handle;
        fn TerminateProcess(p: Handle, code: u32) -> i32;
    }
    static FILE: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);
    static mut PATH: super::FixedBuf<96> = super::FixedBuf::new();
    static PATH_SET: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
    pub unsafe fn open_write(y: u16, mo: u16, d: u16, h: u16, mi: u16, s: u16, text: &[u8]) {
        use core::fmt::Write as _;
        CreateDirectoryA(b"crashlogs\0".as_ptr(), core::ptr::null_mut());
        let p = &mut *core::ptr::addr_of_mut!(PATH);
        let _ = write!(p, "crashlogs\\crash-{y:04}{mo:02}{d:02}-{h:02}{mi:02}{s:02}-{}.log\0", GetCurrentProcessId());
        // GENERIC_WRITE, FILE_SHARE_READ | FILE_SHARE_DELETE, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL
        let f = CreateFileA(p.b.as_ptr(), 0x4000_0000, 0x1 | 0x4, core::ptr::null_mut(), 2, 0x80, core::ptr::null_mut());
        PATH_SET.store(true, core::sync::atomic::Ordering::SeqCst);
        if f as isize == -1 {
            return;
        }
        FILE.store(f as usize, core::sync::atomic::Ordering::SeqCst);
        append(text);
    }
    pub unsafe fn append(text: &[u8]) {
        let f = FILE.load(core::sync::atomic::Ordering::SeqCst) as Handle;
        if f.is_null() {
            return;
        }
        let mut w = 0u32;
        WriteFile(f, text.as_ptr(), text.len() as u32, &mut w, core::ptr::null_mut());
        FlushFileBuffers(f);
    }
    pub fn close() {
        let f = FILE.swap(0, core::sync::atomic::Ordering::SeqCst) as Handle;
        if !f.is_null() {
            unsafe { CloseHandle(f) };
        }
    }
    /// The stage-1 file's path (without the NUL), once stage 1 ran.
    pub fn path() -> Option<&'static str> {
        if !PATH_SET.load(core::sync::atomic::Ordering::SeqCst) {
            return None;
        }
        let s = unsafe { (*core::ptr::addr_of!(PATH)).as_str() };
        Some(s.trim_end_matches('\0'))
    }
    pub unsafe fn exit_now(code: i32) -> ! {
        TerminateProcess(GetCurrentProcess(), code as u32);
        loop {}
    }
}

#[cfg(unix)]
mod raw {
    extern "C" {
        fn mkdir(path: *const u8, mode: u32) -> i32;
        fn open(path: *const u8, flags: i32, mode: u32) -> i32;
        fn write(fd: i32, b: *const u8, n: usize) -> isize;
        fn fsync(fd: i32) -> i32;
        fn close(fd: i32) -> i32;
        fn getpid() -> i32;
        fn _exit(code: i32) -> !;
    }
    static FILE: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(-1);
    static mut PATH: super::FixedBuf<96> = super::FixedBuf::new();
    static PATH_SET: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
    pub unsafe fn open_write(y: u16, mo: u16, d: u16, h: u16, mi: u16, s: u16, text: &[u8]) {
        use core::fmt::Write as _;
        mkdir(b"crashlogs\0".as_ptr(), 0o755);
        let p = &mut *core::ptr::addr_of_mut!(PATH);
        let _ = write!(p, "crashlogs/crash-{y:04}{mo:02}{d:02}-{h:02}{mi:02}{s:02}-{}.log\0", getpid());
        // O_WRONLY | O_CREAT | O_TRUNC (macOS numbers O_CREAT/O_TRUNC differently)
        #[cfg(target_os = "macos")]
        let flags = 0x1 | 0x200 | 0x400;
        #[cfg(not(target_os = "macos"))]
        let flags = 0o1 | 0o100 | 0o1000;
        let f = open(p.b.as_ptr(), flags, 0o644);
        PATH_SET.store(true, core::sync::atomic::Ordering::SeqCst);
        if f < 0 {
            return;
        }
        FILE.store(f, core::sync::atomic::Ordering::SeqCst);
        append(text);
    }
    pub unsafe fn append(text: &[u8]) {
        let f = FILE.load(core::sync::atomic::Ordering::SeqCst);
        if f < 0 {
            return;
        }
        let mut t = text;
        while !t.is_empty() {
            let n = write(f, t.as_ptr(), t.len());
            if n <= 0 {
                break;
            }
            t = &t[n as usize..];
        }
        fsync(f);
    }
    pub fn close() {
        let f = FILE.swap(-1, core::sync::atomic::Ordering::SeqCst);
        if f >= 0 {
            unsafe { close(f) };
        }
    }
    pub fn path() -> Option<&'static str> {
        if !PATH_SET.load(core::sync::atomic::Ordering::SeqCst) {
            return None;
        }
        let s = unsafe { (*core::ptr::addr_of!(PATH)).as_str() };
        Some(s.trim_end_matches('\0'))
    }
    pub unsafe fn exit_now(code: i32) -> ! {
        _exit(code)
    }
}

/// Stage 2 hook for the crash test: fault again while the symbolised report is being built.
fn maybe_refault_test() {
    if REFAULT_TEST.load(std::sync::atomic::Ordering::Relaxed) {
        unsafe {
            let p = core::hint::black_box(16usize) as *const u32;
            let _ = core::ptr::read_volatile(p);
        }
    }
}

fn install_common() {
    CRASH_STACK.store(
        std::env::var("MELEE_CRASH_STACK").ok().and_then(|v| v.parse::<usize>().ok()).unwrap_or(24).clamp(1, 1000),
        std::sync::atomic::Ordering::Relaxed,
    );
    EXE_BASE.store(exe_base(), std::sync::atomic::Ordering::Relaxed);
}

#[cfg(windows)]
fn exe_base() -> usize {
    extern "system" {
        fn GetModuleHandleW(name: *const u16) -> *mut core::ffi::c_void;
    }
    unsafe { GetModuleHandleW(core::ptr::null()) as usize }
}

#[cfg(unix)]
fn exe_base() -> usize {
    #[repr(C)]
    struct DlInfo {
        fname: *const u8,
        fbase: *mut core::ffi::c_void,
        sname: *const u8,
        saddr: *mut core::ffi::c_void,
    }
    extern "C" {
        fn dladdr(addr: *const core::ffi::c_void, info: *mut DlInfo) -> i32;
    }
    let mut i = DlInfo { fname: core::ptr::null(), fbase: core::ptr::null_mut(), sname: core::ptr::null(), saddr: core::ptr::null_mut() };
    if unsafe { dladdr(exe_base as *const core::ffi::c_void, &mut i) } != 0 {
        i.fbase as usize
    } else {
        0
    }
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
    install_common();
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
        fn RtlCaptureStackBackTrace(skip: u32, count: u32, frames: *mut *mut core::ffi::c_void, hash: *mut u32) -> u16;
        fn GetCurrentThreadStackLimits(low: *mut usize, high: *mut usize);
    }
    unsafe extern "system" fn filter(p: *mut ExceptionPointers) -> i32 {
        use core::fmt::Write as _;
        let rec = &*(*p).record;
        if FAULT_DEPTH.fetch_add(1, std::sync::atomic::Ordering::SeqCst) > 0 {
            refault(rec.code as i32);
        }
        // nothing below allocates from the game arena (see "fault path without the game arena"): the fault may be the
        // arena's own (a corrupt free list, its spin lock still held)
        SYSTEM_ONLY.store(true, std::sync::atomic::Ordering::SeqCst);
        melee_engine::arena::mark_foreign_thread();
        let base = EXE_BASE.load(std::sync::atomic::Ordering::Relaxed);
        let addr = rec.address as usize;
        let what = &mut *core::ptr::addr_of_mut!(WHAT);
        let _ = match rec.code {
            0xC0000005 => {
                let op = match rec.info[0] {
                    0 => "read of",
                    1 => "write to",
                    8 => "execute at",
                    _ => "access to",
                };
                write!(what, "access violation ({op} 0x{:X})", rec.info[1])
            }
            0xC00000FD => what.write_str("stack overflow"),
            0xC0000094 => what.write_str("integer divide by zero"),
            0xC000001D => what.write_str("illegal instruction"),
            0xC0000409 => what.write_str("fast-fail (abort / stack buffer overrun)"),
            c => write!(what, "exception 0x{c:08X}"),
        };
        let fault = &mut *core::ptr::addr_of_mut!(FAULT);
        let _ = write!(fault, "exception code:  0x{:08X}\nexception addr:  0x{:X} (melee.exe+0x{:X})\n", rec.code, addr, addr.wrapping_sub(base));
        // A call through a null pointer faults at rip 0 and the unwinder cannot walk from there, so the backtrace
        // below is empty. List the code addresses on the top of the faulting stack instead (x64 CONTEXT.Rsp at
        // 0x98): the first is the return address into the caller of the null pointer.
        #[cfg(target_arch = "x86_64")]
        if !(*p).context.is_null() {
            let rsp = *((*p).context as *const u8).add(0x98).cast::<usize>();
            let mut n = 0;
            // MELEE_CRASH_STACK=N: list up to N return addresses (default 24), scanning deeper, to reach the caller
            // of a deep recursion (a model load)
            let want = CRASH_STACK.load(std::sync::atomic::Ordering::Relaxed);
            // never past the top of this thread's stack: a shallow stack (crash-test) ends within the scan, and the
            // read past it faulted again inside the filter, so no report was written at all
            let (mut lo, mut hi) = (0usize, 0usize);
            GetCurrentThreadStackLimits(&mut lo, &mut hi);
            let words = if rsp >= lo && rsp < hi { (hi - rsp) / 8 } else { 0 };
            for i in 0..(512usize).max(want * 16).min(words) {
                let v = *(rsp as *const usize).add(i);
                if v > base && v < base + 0x400_0000 {
                    let _ = writeln!(fault, "stack[{i:3}]:      melee.exe+0x{:X}", v - base);
                    n += 1;
                    if n == want {
                        break;
                    }
                }
            }
        }
        let frames = &mut *core::ptr::addr_of_mut!(FRAMES);
        let n = RtlCaptureStackBackTrace(0, frames.len() as u32, frames.as_mut_ptr().cast(), core::ptr::null_mut()) as usize;
        push_frames(fault, &frames[..n]);
        if !REPORTING.load(std::sync::atomic::Ordering::SeqCst) {
            fault_stage1(what.as_str(), fault.as_str());
        }
        write_report_with("exception", what.as_str(), fault.as_str());
        1 // EXCEPTION_EXECUTE_HANDLER: end the process after the report
    }
    unsafe { SetUnhandledExceptionFilter(Some(filter)) };
}

#[cfg(unix)]
fn install_fault_handler() {
    // SIGSEGV/SIGBUS/SIGILL/SIGFPE/SIGABRT: report from the handler on an alternate stack, then re-raise.
    install_common();
    extern "C" {
        fn signal(sig: i32, handler: usize) -> usize;
        fn raise(sig: i32) -> i32;
        fn sigaltstack(new: *const StackT, old: *mut StackT) -> i32;
        fn backtrace(buf: *mut *mut core::ffi::c_void, n: i32) -> i32;
    }
    #[cfg(target_os = "linux")]
    extern "C" {
        fn sigaction(sig: i32, act: *const SigAction, old: *mut SigAction) -> i32;
    }
    #[repr(C)]
    struct StackT {
        sp: *mut core::ffi::c_void,
        flags: i32,
        size: usize,
    }
    /// glibc's `struct sigaction` (x86_64 and aarch64): handler, 1024-bit mask, flags, restorer.
    #[cfg(target_os = "linux")]
    #[repr(C)]
    struct SigAction {
        handler: usize,
        mask: [u64; 16],
        flags: i32,
        restorer: usize,
    }
    extern "C" fn handler(sig: i32, info: *mut core::ffi::c_void, uctx: *mut core::ffi::c_void) {
        use core::fmt::Write as _;
        unsafe {
            if FAULT_DEPTH.fetch_add(1, std::sync::atomic::Ordering::SeqCst) > 0 {
                refault(128 + sig);
            }
            // see "fault path without the game arena": from here every allocation is the system heap's
            SYSTEM_ONLY.store(true, std::sync::atomic::Ordering::SeqCst);
            melee_engine::arena::mark_foreign_thread();
            let name = match sig {
                11 => "SIGSEGV (invalid memory access)",
                7 => "SIGBUS",
                4 => "SIGILL (illegal instruction)",
                8 => "SIGFPE",
                6 => "SIGABRT",
                _ => "signal",
            };
            let fault = &mut *core::ptr::addr_of_mut!(FAULT);
            let _ = writeln!(fault, "signal: {sig}");
            #[cfg(target_os = "linux")]
            if !info.is_null() && matches!(sig, 11 | 7 | 4 | 8) {
                // siginfo_t: si_signo, si_errno, si_code, then (8-aligned) si_addr
                let a = *(info as *const u8).add(16).cast::<usize>();
                let _ = writeln!(fault, "fault address:   0x{a:X}");
            }
            #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
            if !uctx.is_null() {
                // ucontext_t: uc_flags, uc_link, uc_stack (24 bytes), then mcontext gregs[]; REG_RIP = 16
                let rip = *(uctx as *const u8).add(40 + 16 * 8).cast::<usize>();
                let base = EXE_BASE.load(std::sync::atomic::Ordering::Relaxed);
                let _ = writeln!(fault, "pc:              0x{rip:X} (melee+0x{:X})", rip.wrapping_sub(base));
            }
            let _ = (info, uctx);
            let frames = &mut *core::ptr::addr_of_mut!(FRAMES);
            let n = backtrace(frames.as_mut_ptr().cast(), frames.len() as i32).max(0) as usize;
            push_frames(fault, &frames[..n]);
            if !REPORTING.load(std::sync::atomic::Ordering::SeqCst) {
                fault_stage1(name, fault.as_str());
            }
            write_report_with("exception", name, fault.as_str());
            signal(sig, 0); // SIG_DFL
            raise(sig);
        }
    }
    unsafe {
        let stack = Box::leak(vec![0u8; 256 * 1024].into_boxed_slice());
        let ss = StackT { sp: stack.as_mut_ptr() as *mut _, flags: 0, size: stack.len() };
        sigaltstack(&ss, core::ptr::null_mut());
        // warm glibc's backtrace (its first call loads libgcc_s, which allocates) outside the handler
        let mut warm = [core::ptr::null_mut::<core::ffi::c_void>(); 4];
        backtrace(warm.as_mut_ptr(), 4);
        for sig in [11, 7, 4, 8, 6] {
            #[cfg(target_os = "linux")]
            {
                // SA_SIGINFO | SA_ONSTACK (on the alternate stack, so a stack overflow is reported too) | SA_NODEFER (a
                // second fault inside the handler re-enters it and `refault` ends the process, instead of the kernel
                // killing it before anything is noted)
                let act = SigAction { handler: handler as usize, mask: [0; 16], flags: 0x4 | 0x0800_0000 | 0x4000_0000, restorer: 0 };
                sigaction(sig, &act, core::ptr::null_mut());
            }
            #[cfg(not(target_os = "linux"))]
            signal(sig, handler as usize);
        }
    }
}

/// `melee crash-test panic|fault|abort`: raise a crash on purpose (release smoke test).
/// Crash-handler tests (environment, read by crash-test only):
/// * `MELEE_CRASH_TEST_ARENA=1`: corrupt the game arena's free lists and stay in the game scope before the fault, so
///   any allocation that reaches the arena faults again (the Linux soak's lost reports);
/// * `MELEE_CRASH_TEST_REFAULT=1`: fault a second time inside the handler's symbolised stage.
pub fn crash_test(kind: &str) -> ! {
    eprintln!("melee: crash-test {kind}");
    let arena = std::env::var("MELEE_CRASH_TEST_ARENA").is_ok_and(|v| v == "1");
    REFAULT_TEST.store(std::env::var("MELEE_CRASH_TEST_REFAULT").is_ok_and(|v| v == "1"), std::sync::atomic::Ordering::Relaxed);
    match kind {
        "panic" => panic!("crash-test: deliberate panic"),
        "fault" => unsafe {
            let _g = arena.then(|| {
                // a first arena allocation (reserves it), then every free list points at unmapped memory
                let g = melee_engine::arena::GameScope::enter();
                let warm: Vec<u64> = vec![1; 64];
                core::hint::black_box(&warm);
                melee_engine::arena::debug_corrupt_free_lists();
                g
            });
            if arena {
                // stderr's lock is fine; this line's formatting does not allocate
                eprintln!("melee: crash-test: arena free lists corrupted, game scope on");
            }
            let p = core::hint::black_box(8usize) as *const u32;
            let v = core::ptr::read_volatile(p);
            eprintln!("unreachable {v}");
            std::process::exit(0)
        },
        _ => std::process::abort(),
    }
}

