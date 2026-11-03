//! Opt-in in-process sampling profiler (`MELEE_SAMPLE=<report.txt>`), for machines without admin rights (no ETW/WPR).
//! A helper thread suspends the main thread every ~1 ms, unwinds its stack with the PE unwind tables and counts
//! the frames; at exit the addresses are named through dbghelp (the release profile keeps a PDB) and a flat
//! (self time) and an inclusive table are written. No OS input is involved. Windows only; elsewhere a no-op.

#[cfg(windows)]
mod imp {
    use core::ffi::c_void;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    #[repr(C, align(16))]
    struct Ctx([u8; 1232]);

    extern "system" {
        fn GetCurrentProcess() -> *mut c_void;
        fn GetCurrentThread() -> *mut c_void;
        fn DuplicateHandle(sp: *mut c_void, sh: *mut c_void, tp: *mut c_void, th: *mut *mut c_void, access: u32, inh: i32, opt: u32) -> i32;
        fn SuspendThread(h: *mut c_void) -> u32;
        fn ResumeThread(h: *mut c_void) -> u32;
        fn GetThreadContext(h: *mut c_void, c: *mut Ctx) -> i32;
        fn RtlLookupFunctionEntry(pc: u64, base: *mut u64, hist: *mut c_void) -> *mut c_void;
        fn RtlVirtualUnwind(ty: u32, base: u64, pc: u64, fe: *mut c_void, c: *mut Ctx, hd: *mut *mut c_void, est: *mut u64, cp: *mut c_void) -> *mut c_void;
        fn timeBeginPeriod(ms: u32) -> u32;
        fn SymInitialize(p: *mut c_void, path: *const u8, invade: i32) -> i32;
        fn SymSetOptions(o: u32) -> u32;
        fn GetLastError() -> u32;
        fn SymFromAddr(p: *mut c_void, addr: u64, disp: *mut u64, sym: *mut u8) -> i32;
    }

    pub struct Guard {
        stop: Arc<AtomicBool>,
        join: Option<std::thread::JoinHandle<()>>,
        data: Arc<Mutex<Vec<Vec<u64>>>>,
        out: String,
    }

    pub fn start(out: String) -> Guard {
        let stop = Arc::new(AtomicBool::new(false));
        let data: Arc<Mutex<Vec<Vec<u64>>>> = Arc::new(Mutex::new(Vec::with_capacity(200_000)));
        let mut th: *mut c_void = core::ptr::null_mut();
        unsafe {
            DuplicateHandle(GetCurrentProcess(), GetCurrentThread(), GetCurrentProcess(), &mut th, 0, 0, 2);
            timeBeginPeriod(1);
        }
        let thv = th as usize;
        let (s2, d2) = (stop.clone(), data.clone());
        let join = std::thread::spawn(move || {
            let th = thv as *mut c_void;
            let mut ctx = Box::new(Ctx([0; 1232]));
            let mut frames = [0u64; 48];
            while !s2.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(1));
                // Only while the main thread is inside a gated region (play's Game::tick): suspended in a driver or
                // the loader it can hold the function-table lock RtlLookupFunctionEntry takes, a deadlock at boot.
                if super::GATED.load(Ordering::Relaxed) && !super::IN_GATE.load(Ordering::Acquire) {
                    continue;
                }
                let mut n = 0;
                unsafe {
                    if SuspendThread(th) == u32::MAX {
                        continue;
                    }
                    ctx.0[0x30..0x34].copy_from_slice(&0x10000Bu32.to_le_bytes());
                    if GetThreadContext(th, &mut *ctx) != 0 {
                        while n < frames.len() {
                            let rip = u64::from_le_bytes(ctx.0[0xF8..0x100].try_into().unwrap());
                            if rip == 0 {
                                break;
                            }
                            frames[n] = rip;
                            n += 1;
                            let mut base = 0u64;
                            let fe = RtlLookupFunctionEntry(rip, &mut base, core::ptr::null_mut());
                            if fe.is_null() {
                                let rsp = u64::from_le_bytes(ctx.0[0x98..0xA0].try_into().unwrap());
                                if rsp == 0 {
                                    break;
                                }
                                let ret = *(rsp as *const u64);
                                ctx.0[0xF8..0x100].copy_from_slice(&ret.to_le_bytes());
                                ctx.0[0x98..0xA0].copy_from_slice(&(rsp + 8).to_le_bytes());
                            } else {
                                let mut hd = core::ptr::null_mut();
                                let mut est = 0u64;
                                RtlVirtualUnwind(0, base, rip, fe, &mut *ctx, &mut hd, &mut est, core::ptr::null_mut());
                            }
                        }
                    }
                    ResumeThread(th);
                }
                if n > 0 {
                    d2.lock().unwrap().push(frames[..n].to_vec());
                }
            }
        });
        Guard { stop, join: Some(join), data, out }
    }

    fn demangle(s: &str) -> String {
        // legacy Rust mangling: _ZN <len><seg>... 17h<hash>E
        let Some(rest) = s.strip_prefix("_ZN") else { return s.to_string() };
        let mut out: Vec<String> = Vec::new();
        let b = rest.as_bytes();
        let mut i = 0;
        while i < b.len() && b[i].is_ascii_digit() {
            let mut n = 0;
            while i < b.len() && b[i].is_ascii_digit() {
                n = n * 10 + (b[i] - b'0') as usize;
                i += 1;
            }
            if i + n > b.len() {
                break;
            }
            let seg = &rest[i..i + n];
            i += n;
            if !(seg.starts_with('h') && seg.len() == 17) {
                out.push(seg.replace("$LT$", "<").replace("$GT$", ">").replace("$u20$", " ").replace("$C$", ",").replace("..", "::"));
            }
        }
        out.join("::")
    }

    /// Names for code addresses (dbghelp; the release profile keeps a PDB, `MELEE_SAMPLE_PDB` adds a search path).
    pub fn symbolize(addrs: impl Iterator<Item = u64>) -> HashMap<u64, String> {
        let mut names: HashMap<u64, String> = HashMap::new();
        unsafe {
            let p = GetCurrentProcess();
            SymSetOptions(2);
            let sp = std::env::var("MELEE_SAMPLE_PDB").unwrap_or_default() + "\0";
            SymInitialize(p, if sp.len() > 1 { sp.as_ptr() } else { core::ptr::null() }, 1);
            let mut buf = vec![0u8; 88 + 1024];
            let mut name = |a: u64| -> String {
                buf.iter_mut().for_each(|x| *x = 0);
                buf[0..4].copy_from_slice(&88u32.to_le_bytes());
                buf[80..84].copy_from_slice(&1000u32.to_le_bytes());
                let mut disp = 0u64;
                if SymFromAddr(p, a, &mut disp, buf.as_mut_ptr()) != 0 {
                    let nl = u32::from_le_bytes(buf[76..80].try_into().unwrap()) as usize;
                    let s = String::from_utf8_lossy(&buf[84..84 + nl.min(1000)]).to_string();
                    demangle(&s)
                } else {
                    if std::env::var_os("MELEE_SAMPLE_DBG").is_some() { eprintln!("SymFromAddr {a:#x}: {}", GetLastError()); }
                    format!("{a:#x}")
                }
            };
            for a in addrs {
                names.entry(a).or_insert_with(|| name(a));
            }
        }
        names
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
            if let Some(j) = self.join.take() {
                let _ = j.join();
            }
            let data = std::mem::take(&mut *self.data.lock().unwrap());
            let total = data.len();
            let names = symbolize(data.iter().flatten().copied());
            let data: Vec<Vec<u64>> = match std::env::var("MELEE_SAMPLE_ROOT") {
                Ok(r) => data.into_iter().filter(|s| s.iter().any(|a| names[a].contains(&r))).collect(),
                Err(_) => data,
            };
            let total = data.len();
            let mut flat: HashMap<&str, usize> = HashMap::new();
            let mut incl: HashMap<&str, usize> = HashMap::new();
            for s in &data {
                *flat.entry(names[&s[0]].as_str()).or_default() += 1;
                let mut seen = std::collections::HashSet::new();
                for a in s {
                    let n = names[a].as_str();
                    if seen.insert(n) {
                        *incl.entry(n).or_default() += 1;
                    }
                }
            }
            let mut rep = format!("samples {total}\n\n== self time ==\n");
            let mut v: Vec<_> = flat.into_iter().collect();
            v.sort_by(|a, b| b.1.cmp(&a.1));
            for (n, c) in v.iter().take(60) {
                rep += &format!("{:6.2}% {:7} {}\n", *c as f64 * 100.0 / total as f64, c, n);
            }
            rep += "\n== inclusive ==\n";
            let mut v: Vec<_> = incl.into_iter().collect();
            v.sort_by(|a, b| b.1.cmp(&a.1));
            for (n, c) in v.iter().take(120) {
                rep += &format!("{:6.2}% {:7} {}\n", *c as f64 * 100.0 / total as f64, c, n);
            }
            match std::fs::write(&self.out, rep) {
                Ok(()) => eprintln!("prof: {total} samples -> {}", self.out),
                Err(e) => eprintln!("prof: {}: {e}", self.out),
            }
        }
    }
}

#[cfg(windows)]
pub use imp::{symbolize, Guard};

use std::sync::atomic::{AtomicBool, Ordering};
/// MELEE_SAMPLE_GATE (default on in `play`): sample only between [`gate`]`(true)` and `gate(false)`.
static GATED: AtomicBool = AtomicBool::new(false);
static IN_GATE: AtomicBool = AtomicBool::new(false);

/// Turn gating on (samples are then taken only inside [`gate`] regions).
pub fn set_gated(on: bool) {
    GATED.store(on, Ordering::Relaxed);
}

/// Enter / leave a sampled region (cheap: one atomic store).
#[inline]
pub fn gate(inside: bool) {
    IN_GATE.store(inside, Ordering::Release);
}

#[cfg(windows)]
pub fn from_env() -> Option<Guard> {
    std::env::var("MELEE_SAMPLE").ok().map(imp::start)
}

#[cfg(not(windows))]
pub struct Guard;
#[cfg(not(windows))]
pub fn from_env() -> Option<Guard> {
    None
}
