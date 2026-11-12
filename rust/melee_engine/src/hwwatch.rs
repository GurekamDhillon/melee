//! Debug: hardware write watchpoints on the decoder's "non-zero word" issue strings (`MELEE_HWWATCH_ISSUE=1`).
//! Every kept issue string starting with that text gets an 8-byte write watch on its first word (debug registers
//! of the thread that decodes, round robin over DR0..DR3); a write prints the value, the watched address and a
//! backtrace. Found the in-place ItCo decode's stray 0x113 write (zero-hitch.md, asset service slice 4).

#[cfg(windows)]
mod imp {
    use core::ffi::c_void;
    use core::sync::atomic::{AtomicUsize, Ordering::*};

    #[repr(C, align(16))]
    struct Ctx([u8; 0x4D0]);
    extern "system" {
        fn GetCurrentThread() -> *mut c_void;
        fn GetThreadContext(h: *mut c_void, c: *mut Ctx) -> i32;
        fn SetThreadContext(h: *mut c_void, c: *const Ctx) -> i32;
        fn AddVectoredExceptionHandler(first: u32, h: unsafe extern "system" fn(*mut c_void) -> i32) -> *mut c_void;
        fn RtlCaptureStackBackTrace(skip: u32, n: u32, out: *mut *mut c_void, hash: *mut u32) -> u16;
    }
    const NF: usize = 28;
    const NH: usize = 16;
    // hits recorded by the handler (no allocation, no locks there), printed by `flush` from normal code
    static mut HIT: [(usize, u32, usize, [usize; NF]); NH] = [(0, 0, 0, [0; NF]); NH];
    static NREC: AtomicUsize = AtomicUsize::new(0);
    static NPRINTED: AtomicUsize = AtomicUsize::new(0);
    const FLAGS: usize = 0x30;
    const DR0: usize = 0x48;
    const DR6: usize = 0x68;
    const DR7: usize = 0x70;
    const CONTEXT_DEBUG_REGISTERS: u32 = 0x0010_0010;

    static NEXT: AtomicUsize = AtomicUsize::new(0);
    static ADDR: [AtomicUsize; 4] = [const { AtomicUsize::new(0) }; 4];
    static HITS: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "system" fn handler(info: *mut c_void) -> i32 {
        let rec = *(info as *const *const u8);
        if *(rec as *const u32) != 0x8000_0004 {
            return 0;
        }
        let ctx = *(info as *const *mut u8).add(1);
        let dr6 = *(ctx.add(DR6) as *const u64);
        if dr6 & 0xF == 0 {
            return 0;
        }
        *(ctx.add(DR6) as *mut u64) = 0;
        HITS.fetch_add(1, SeqCst);
        let n = NREC.load(SeqCst);
        let k0 = (0..4).find(|k| dr6 & (1 << k) != 0).unwrap_or(0);
        if n < NH && *(ADDR[k0].load(SeqCst) as *const u32) == 0x113 {
            let k = (0..4).find(|k| dr6 & (1 << k) != 0).unwrap_or(0);
            let a = ADDR[k].load(SeqCst);
            let h = &mut *core::ptr::addr_of_mut!(HIT[n]);
            h.0 = a;
            h.1 = *(a as *const u32);
            h.2 = *(ctx.add(0xF8) as *const u64) as usize;
            RtlCaptureStackBackTrace(0, NF as u32, h.3.as_mut_ptr() as *mut *mut c_void, core::ptr::null_mut());
            NREC.store(n + 1, SeqCst);
        }
        -1
    }

    /// Print the hits recorded since the last call (normal code: symbol lookup allocates).
    pub fn flush(tag: &str) {
        let n = NREC.load(SeqCst);
        for i in NPRINTED.swap(n, SeqCst)..n {
            let h = unsafe { &*core::ptr::addr_of!(HIT[i]) };
            let mut s = format!("hwwatch [{tag}]: write to watched issue word {:#x}: now {:#010x}, rip {:#x}", h.0, h.1, h.2);
            for &f in core::iter::once(&h.2).chain(h.3.iter()) {
                if f == 0 { break; }
                backtrace::resolve(f as *mut c_void, |sym| {
                    s += &format!("
    {f:#x} {}", sym.name().map(|n| n.to_string()).unwrap_or_default());
                });
            }
            eprintln!("{s}");
        }
    }

    static SEEN: AtomicUsize = AtomicUsize::new(0);
    static NTH: AtomicUsize = AtomicUsize::new(usize::MAX);

    /// Decode end: report kept issue strings whose text no longer starts as written (index among the decode's
    /// "non-zero word" issues, counted over the whole run).
    pub fn check_issues(v: &[String]) {
        if !ON.load(SeqCst) {
            return;
        }
        let base = SEEN.load(SeqCst) - v.iter().filter(|s| s.as_bytes().get(4..13) == Some(b"zero word")).count();
        let mut k = base;
        for s in v {
            let b = s.as_bytes();
            if b.get(4..13) == Some(b"zero word") {
                if &b[..4] != b"non-" {
                    eprintln!("hwwatch: issue #{k} overwritten: first word {:#010x}, text {:?}", u32::from_le_bytes(b[..4].try_into().unwrap()), &s[4..]);
                }
                k += 1;
            }
        }
    }
    static ON: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

    fn hook(p: *const u8, len: usize) {
        flush("issue");
        unsafe {
            if len < 13 || core::slice::from_raw_parts(p, 13) != b"non-zero word" {
                return;
            }
            let i = SEEN.fetch_add(1, SeqCst);
            let nth = NTH.load(SeqCst);
            if nth != usize::MAX && i != nth {
                return;
            }
            let k = NEXT.fetch_add(1, SeqCst) % 4;
            ADDR[k].store(p as usize, SeqCst);
            let mut c = Ctx([0; 0x4D0]);
            *(c.0.as_mut_ptr().add(FLAGS) as *mut u32) = CONTEXT_DEBUG_REGISTERS;
            let t = GetCurrentThread();
            if GetThreadContext(t, &mut c) == 0 {
                return;
            }
            *(c.0.as_mut_ptr().add(DR0 + 8 * k) as *mut u64) = p as u64;
            let mut d7 = *(c.0.as_ptr().add(DR7) as *const u64);
            d7 &= !(0xF << (16 + 4 * k));
            d7 |= (1 << (2 * k)) | (0b1001 << (16 + 4 * k)); // local enable, write, 8 bytes
            *(c.0.as_mut_ptr().add(DR7) as *mut u64) = d7;
            *(c.0.as_mut_ptr().add(FLAGS) as *mut u32) = CONTEXT_DEBUG_REGISTERS;
            SetThreadContext(t, &c);
        }
    }

    pub fn install() {
        if std::env::var_os("MELEE_HWWATCH_ISSUE").is_none() {
            return;
        }
        ON.store(true, SeqCst);
        if let Some(n) = std::env::var("MELEE_HWWATCH_NTH").ok().and_then(|v| v.parse().ok()) {
            NTH.store(n, SeqCst);
        }
        unsafe { AddVectoredExceptionHandler(1, handler) };
        melee_core::gcdecode::ISSUE_HOOK.store(hook as fn(*const u8, usize) as usize, SeqCst);
        eprintln!("hwwatch: watching decoder issue strings");
    }
}

#[cfg(windows)]
pub use imp::{check_issues, flush, install};
#[cfg(not(windows))]
pub fn install() {}
#[cfg(not(windows))]
pub fn flush(_: &str) {}
#[cfg(not(windows))]
pub fn check_issues(_: &[String]) {}
