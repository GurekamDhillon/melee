//! Copy-on-first-write capture of a big, mostly cold memory range (the game arena after boot: ~5 GB of decoded
//! archive objects that the game reads and almost never writes). A rollback base would otherwise be a second copy
//! of all of it. Instead the range is made read-only; the first write to a page raises an access violation that a
//! vectored handler answers by saving the page's old contents into a side store and making the page writable again.
//! A page that was never written still holds its base contents in place.
//!
//! The handler runs inside whatever code faulted (possibly the allocator): it takes no locks and allocates nothing
//! (the store and the bitmap are reserved before protection; `VirtualAlloc` commits store pages).

#[cfg(windows)]
mod imp {
    use core::ffi::c_void;
    const PAGE: usize = 4096;

    extern "system" {
        fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
        fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
        fn AddVectoredExceptionHandler(first: u32, h: unsafe extern "system" fn(*mut c_void) -> i32) -> *mut c_void;
        fn RemoveVectoredExceptionHandler(h: *mut c_void) -> u32;
    }
    const MEM_COMMIT: u32 = 0x1000;
    const MEM_RESERVE: u32 = 0x2000;
    const PAGE_READWRITE: u32 = 0x04;
    const PAGE_READONLY: u32 = 0x02;

    static mut START: usize = 0;
    static mut LEN: usize = 0;
    static mut STORE: usize = 0;
    static mut BITS: *mut u64 = core::ptr::null_mut();
    static mut HANDLER: *mut c_void = core::ptr::null_mut();
    static mut FAULTS: u64 = 0;
    /// The store's reserved size and the bitmap's length in words: both are kept from one `protect` to the next
    /// (each rollback base, i.e. each match, protects the arena again; a fresh store per base, never freed, was the
    /// ~70 MB-per-match growth of a long session: the pages the match wrote stayed committed in the old store).
    static mut STORE_CAP: usize = 0;
    static mut BITS_WORDS: usize = 0;
    const MEM_DECOMMIT: u32 = 0x4000;
    const MEM_RELEASE: u32 = 0x8000;
    extern "system" {
        fn VirtualFree(addr: *mut c_void, size: usize, ty: u32) -> i32;
    }

    unsafe extern "system" fn handler(info: *mut c_void) -> i32 {
        let rec = *(info as *const *const u8);
        let code = *(rec as *const u32);
        if code != 0xC000_0005 {
            return 0;
        }
        let nparams = *(rec.add(24) as *const u32);
        if nparams < 2 {
            return 0;
        }
        let is_write = *(rec.add(32) as *const usize) == 1;
        let addr = *(rec.add(40) as *const usize);
        if !is_write || START == 0 || addr < START || addr >= START + LEN {
            return 0;
        }
        let page = addr & !(PAGE - 1);
        let idx = (page - START) / PAGE;
        let bit = 1u64 << (idx % 64);
        // save the old contents once, then let the write through
        if *BITS.add(idx / 64) & bit == 0 {
            let dst = (STORE + (page - START)) as *mut c_void;
            if VirtualAlloc(dst, PAGE, MEM_COMMIT, PAGE_READWRITE).is_null() {
                return 0;
            }
            core::ptr::copy_nonoverlapping(page as *const u8, dst as *mut u8, PAGE);
            *BITS.add(idx / 64) |= bit;
            FAULTS += 1;
        }
        let mut old = 0u32;
        if VirtualProtect(page as *mut c_void, PAGE, PAGE_READWRITE, &mut old) == 0 {
            return 0;
        }
        -1 // EXCEPTION_CONTINUE_EXECUTION
    }

    /// Make `[start, start+len)` copy-on-first-write. `len` is a multiple of the page size. At most one range at a time.
    pub unsafe fn protect(start: usize, len: usize) -> Result<(), String> {
        if START != 0 {
            return Err("a copy-on-write range is already active".into());
        }
        let words = (len / PAGE).div_ceil(64);
        if BITS.is_null() || BITS_WORDS < words {
            // grows only with the arena; the old bitmap is freed (no handler is installed while START == 0)
            if !BITS.is_null() {
                drop(Box::from_raw(core::ptr::slice_from_raw_parts_mut(BITS, BITS_WORDS)));
            }
            BITS = Box::leak(vec![0u64; words].into_boxed_slice()).as_mut_ptr();
            BITS_WORDS = words;
        } else {
            core::ptr::write_bytes(BITS, 0, words);
        }
        if STORE == 0 || STORE_CAP < len {
            if STORE != 0 {
                VirtualFree(STORE as *mut c_void, 0, MEM_RELEASE);
            }
            let store = VirtualAlloc(core::ptr::null_mut(), len, MEM_RESERVE, PAGE_READWRITE);
            if store.is_null() {
                STORE = 0;
                STORE_CAP = 0;
                return Err("cannot reserve the copy-on-write store".into());
            }
            STORE = store as usize;
            STORE_CAP = len;
        }
        START = start;
        LEN = len;
        HANDLER = AddVectoredExceptionHandler(1, handler);
        let mut old = 0u32;
        if VirtualProtect(start as *mut c_void, len, PAGE_READONLY, &mut old) == 0 {
            release();
            return Err("VirtualProtect failed".into());
        }
        Ok(())
    }

    /// Undo `protect` (the range becomes writable; the store's pages are decommitted, its reservation kept for the
    /// next `protect`).
    pub unsafe fn release() {
        if START == 0 {
            return;
        }
        let mut old = 0u32;
        VirtualProtect(START as *mut c_void, LEN, PAGE_READWRITE, &mut old);
        if !HANDLER.is_null() {
            RemoveVectoredExceptionHandler(HANDLER);
            HANDLER = core::ptr::null_mut();
        }
        // the saved pages are of no use once the base is gone: give them back (the reservation is reused)
        // (the whole reservation: a reserved-but-uncommitted page decommits as a no-op)
        VirtualFree(STORE as *mut c_void, STORE_CAP, MEM_DECOMMIT);
        START = 0;
    }

    /// The saved base page at `page_addr`, when it was written since `protect`.
    pub unsafe fn saved(page_addr: usize) -> Option<&'static [u8]> {
        if START == 0 || page_addr < START || page_addr >= START + LEN {
            return None;
        }
        let idx = (page_addr - START) / PAGE;
        if *BITS.add(idx / 64) & (1u64 << (idx % 64)) != 0 {
            Some(core::slice::from_raw_parts((STORE + (page_addr - START)) as *const u8, PAGE))
        } else {
            None
        }
    }

    /// OR the written-page bitmap (bit i = page i of the range, ever written since `protect`) into `out`.
    pub fn or_written(out: &mut Vec<u64>) {
        unsafe {
            if START == 0 {
                return;
            }
            let words = (LEN / PAGE).div_ceil(64);
            if out.len() < words {
                out.resize(words, 0);
            }
            for (i, w) in out[..words].iter_mut().enumerate() {
                *w |= *BITS.add(i);
            }
        }
    }

    pub fn active() -> bool {
        unsafe { START != 0 }
    }
    pub fn faults() -> u64 {
        unsafe { FAULTS }
    }
}

#[cfg(windows)]
pub use imp::*;

/// A copied-in-the-background base of one more range (the host block, 64 MB): at capture the range goes read-only and
/// a helper thread copies it page by page into the base buffer, making each copied run writable again; a write that
/// gets to a page first faults, and the handler copies that page itself (or waits for the helper's copy of it) before
/// letting the write through. The capture costs one VirtualProtect instead of a 64 MB copy on the game thread.
/// Per page: 0 = not copied, 1 = being copied, 2 = copied (the base buffer holds the page as it was at capture).
/// A page that is not yet 2 was not written since the capture: its live bytes are its base bytes.
#[cfg(windows)]
pub mod bg {
    use core::ffi::c_void;
    use core::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering::*};
    const PAGE: usize = 4096;
    /// Pages per helper step (one VirtualProtect each).
    const RUN: usize = 64;

    extern "system" {
        fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
        fn AddVectoredExceptionHandler(first: u32, h: unsafe extern "system" fn(*mut c_void) -> i32) -> *mut c_void;
        fn RemoveVectoredExceptionHandler(h: *mut c_void) -> u32;
    }
    const PAGE_READWRITE: u32 = 0x04;
    const PAGE_READONLY: u32 = 0x02;

    static START: AtomicUsize = AtomicUsize::new(0);
    static LEN: AtomicUsize = AtomicUsize::new(0);
    static DST: AtomicUsize = AtomicUsize::new(0);
    static STATE: AtomicUsize = AtomicUsize::new(0);
    static STOP: AtomicBool = AtomicBool::new(false);
    static FAULTS: AtomicUsize = AtomicUsize::new(0);
    static mut HANDLER: *mut c_void = core::ptr::null_mut();
    static mut THREAD: Option<std::thread::JoinHandle<()>> = None;
    static mut STATE_BUF: Vec<AtomicU8> = Vec::new();

    unsafe fn state(i: usize) -> &'static AtomicU8 {
        &*(STATE.load(Relaxed) as *const AtomicU8).add(i)
    }

    /// Copy page `i` into the base buffer unless someone did (or is doing) it; returns once it is copied.
    unsafe fn claim_copy(i: usize) {
        let st = state(i);
        loop {
            match st.compare_exchange(0, 1, Acquire, Acquire) {
                Ok(_) => {
                    let (s, d) = (START.load(Relaxed), DST.load(Relaxed));
                    core::ptr::copy_nonoverlapping((s + i * PAGE) as *const u8, (d + i * PAGE) as *mut u8, PAGE);
                    st.store(2, Release);
                    return;
                }
                Err(2) => return,
                Err(_) => core::hint::spin_loop(),
            }
        }
    }

    unsafe extern "system" fn handler(info: *mut c_void) -> i32 {
        let rec = *(info as *const *const u8);
        if *(rec as *const u32) != 0xC000_0005 || *(rec.add(24) as *const u32) < 2 {
            return 0;
        }
        let is_write = *(rec.add(32) as *const usize) == 1;
        let addr = *(rec.add(40) as *const usize);
        let (s, l) = (START.load(Acquire), LEN.load(Relaxed));
        if !is_write || s == 0 || addr < s || addr >= s + l {
            return 0;
        }
        let i = (addr - s) / PAGE;
        claim_copy(i);
        FAULTS.fetch_add(1, Relaxed);
        let mut old = 0u32;
        if VirtualProtect((s + i * PAGE) as *mut c_void, PAGE, PAGE_READWRITE, &mut old) == 0 {
            return 0;
        }
        -1
    }

    /// Start: `[start, start+len)` read-only, its base copied into `dst` (len bytes, already committed) by a helper.
    pub unsafe fn begin(start: usize, len: usize, dst: *mut u8) -> Result<(), String> {
        if START.load(Relaxed) != 0 {
            return Err("a background base copy is already active".into());
        }
        let pages = len / PAGE;
        // one per-page state buffer, kept and cleared from one base to the next (a fresh leaked one per begin was
        // ~16 KB per match); no helper or handler touches it while START == 0
        let buf = &mut *core::ptr::addr_of_mut!(STATE_BUF);
        if buf.len() < pages {
            let _s = crate::arena::SysScope::enter(); // the system heap: a rollback load never rewinds it
            *buf = Vec::from_iter((0..pages).map(|_| AtomicU8::new(0)));
        } else {
            buf[..pages].iter().for_each(|a| a.store(0, Relaxed));
        }
        STATE.store(buf.as_ptr() as usize, Relaxed);
        DST.store(dst as usize, Relaxed);
        LEN.store(len, Relaxed);
        STOP.store(false, Relaxed);
        FAULTS.store(0, Relaxed);
        START.store(start, Release);
        HANDLER = AddVectoredExceptionHandler(1, handler);
        let mut old = 0u32;
        if VirtualProtect(start as *mut c_void, len, PAGE_READONLY, &mut old) == 0 {
            release_copy();
            return Err("VirtualProtect failed".into());
        }
        let _s = crate::arena::SysScope::enter();
        THREAD = std::thread::Builder::new()
            .name("snapshot-base-copy".into())
            .spawn(move || {
                let mut i = 0;
                while i < pages && !STOP.load(Relaxed) {
                    let n = RUN.min(pages - i);
                    for k in i..i + n {
                        claim_copy(k);
                    }
                    let mut old = 0u32;
                    VirtualProtect((start + i * PAGE) as *mut c_void, n * PAGE, PAGE_READWRITE, &mut old);
                    i += n;
                }
            })
            .ok();
        if THREAD.is_none() {
            // no helper: the handler alone copies (pages never written stay read-only until `end`)
        }
        Ok(())
    }

    /// The base copy of page `i` is in the buffer (else the live page is still the base page).
    #[inline]
    pub fn copied(i: usize) -> bool {
        let s = START.load(Relaxed);
        s != 0 && i < LEN.load(Relaxed) / PAGE && unsafe { state(i).load(Acquire) == 2 }
    }

    pub fn active() -> bool {
        START.load(Relaxed) != 0
    }

    pub fn faults() -> usize {
        FAULTS.load(Relaxed)
    }

    /// Wait for the helper to finish (every page copied, the range writable again).
    pub fn finish() {
        unsafe {
            if let Some(t) = THREAD.take() {
                let _ = t.join();
            }
        }
    }

    /// Stop: the helper ends, every page is copied, the range writable and the handler gone. The base buffer is then
    /// complete (the caller keeps using it).
    pub unsafe fn release_copy() {
        let s = START.load(Relaxed);
        if s == 0 {
            return;
        }
        finish();
        let l = LEN.load(Relaxed);
        for i in 0..l / PAGE {
            claim_copy(i);
        }
        let mut old = 0u32;
        VirtualProtect(s as *mut c_void, l, PAGE_READWRITE, &mut old);
        if !HANDLER.is_null() {
            RemoveVectoredExceptionHandler(HANDLER);
            HANDLER = core::ptr::null_mut();
        }
        START.store(0, Release);
    }
}

/// Linux/Unix: the same scheme with `mprotect` and a SIGSEGV handler. The range stays readable, so any fault
/// inside it is a write. The store is a lazily-committed anonymous mapping (no allocation in the handler). A fault
/// outside the range puts the previous handler back and returns, so the faulting instruction re-runs under it
/// (Rust's stack-overflow guard, a debugger, the default action).
#[cfg(unix)]
mod imp_unix {
    use core::ffi::c_void;
    const PAGE: usize = 4096;

    static mut START: usize = 0;
    static mut LEN: usize = 0;
    static mut STORE: usize = 0;
    static mut BITS: *mut u64 = core::ptr::null_mut();
    static mut FAULTS: u64 = 0;
    static mut OLD: core::mem::MaybeUninit<libc::sigaction> = core::mem::MaybeUninit::uninit();
    /// The store's mapped size and the bitmap's length in words, kept from one `protect` to the next (see the Windows
    /// side: a fresh store per rollback base, never unmapped, grew the process by the pages each match wrote).
    static mut STORE_CAP: usize = 0;
    static mut BITS_WORDS: usize = 0;
    static mut INSTALLED: bool = false;

    unsafe extern "C" fn handler(_sig: i32, info: *mut libc::siginfo_t, _ctx: *mut c_void) {
        let addr = (*info).si_addr() as usize;
        // the host block's background base copy (`bg`) shares this handler
        #[cfg(target_os = "linux")]
        if super::bg::on_fault(addr) {
            return;
        }
        if START == 0 || addr < START || addr >= START + LEN {
            // not ours: hand the signal back to whoever had it
            libc::sigaction(libc::SIGSEGV, OLD.as_ptr(), core::ptr::null_mut());
            INSTALLED = false;
            return;
        }
        let page = addr & !(PAGE - 1);
        let idx = (page - START) / PAGE;
        let bit = 1u64 << (idx % 64);
        if *BITS.add(idx / 64) & bit == 0 {
            core::ptr::copy_nonoverlapping(page as *const u8, (STORE + (page - START)) as *mut u8, PAGE);
            *BITS.add(idx / 64) |= bit;
            FAULTS += 1;
        }
        libc::mprotect(page as *mut c_void, PAGE, libc::PROT_READ | libc::PROT_WRITE);
    }

    /// Make `[start, start+len)` copy-on-first-write. `len` is a multiple of the page size. At most one range at a time.
    pub unsafe fn protect(start: usize, len: usize) -> Result<(), String> {
        if START != 0 {
            return Err("a copy-on-write range is already active".into());
        }
        let words = (len / PAGE).div_ceil(64);
        if BITS.is_null() || BITS_WORDS < words {
            // grows only with the arena; the old bitmap (a few hundred KB at most) is left as it was
            BITS = Box::leak(vec![0u64; words].into_boxed_slice()).as_mut_ptr();
            BITS_WORDS = words;
        } else {
            core::ptr::write_bytes(BITS, 0, words);
        }
        if STORE == 0 || STORE_CAP < len {
            if STORE != 0 {
                libc::munmap(STORE as *mut c_void, STORE_CAP);
            }
            let store = libc::mmap(
                core::ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE,
                -1,
                0,
            );
            if store == libc::MAP_FAILED {
                STORE = 0;
                STORE_CAP = 0;
                return Err("cannot reserve the copy-on-write store".into());
            }
            STORE = store as usize;
            STORE_CAP = len;
        }
        START = start;
        LEN = len;
        install()?;
        if libc::mprotect(start as *mut c_void, len, libc::PROT_READ) != 0 {
            return Err("mprotect failed".into());
        }
        Ok(())
    }

    /// Undo `protect` (the range becomes writable; the store's pages are dropped, its mapping kept for the next
    /// `protect`).
    pub unsafe fn release() {
        if START == 0 {
            return;
        }
        libc::mprotect(START as *mut c_void, LEN, libc::PROT_READ | libc::PROT_WRITE);
        START = 0;
        uninstall_if_idle();
        // the saved pages are of no use once the base is gone: give them back (the mapping is reused)
        libc::madvise(STORE as *mut c_void, LEN, libc::MADV_DONTNEED);
    }

    /// Install the SIGSEGV handler unless it is in place (one handler for the arena range and the host block's
    /// background copy, `bg`).
    pub(crate) unsafe fn install() -> Result<(), String> {
        if !INSTALLED {
            let mut sa: libc::sigaction = core::mem::zeroed();
            sa.sa_sigaction = handler as usize;
            sa.sa_flags = libc::SA_SIGINFO | libc::SA_ONSTACK;
            libc::sigemptyset(&mut sa.sa_mask);
            if libc::sigaction(libc::SIGSEGV, &sa, OLD.as_mut_ptr()) != 0 {
                return Err("sigaction failed".into());
            }
            INSTALLED = true;
        }
        Ok(())
    }

    /// Put the previous SIGSEGV handler back once neither the arena range nor the background copy is active.
    pub(crate) unsafe fn uninstall_if_idle() {
        #[cfg(target_os = "linux")]
        if super::bg::active() {
            return;
        }
        if INSTALLED && START == 0 {
            libc::sigaction(libc::SIGSEGV, OLD.as_ptr(), core::ptr::null_mut());
            INSTALLED = false;
        }
    }

    /// The saved base page at `page_addr`, when it was written since `protect`.
    pub unsafe fn saved(page_addr: usize) -> Option<&'static [u8]> {
        if START == 0 || page_addr < START || page_addr >= START + LEN {
            return None;
        }
        let idx = (page_addr - START) / PAGE;
        if *BITS.add(idx / 64) & (1u64 << (idx % 64)) != 0 {
            Some(core::slice::from_raw_parts((STORE + (page_addr - START)) as *const u8, PAGE))
        } else {
            None
        }
    }

    /// OR the written-page bitmap (bit i = page i of the range, ever written since `protect`) into `out`.
    pub fn or_written(out: &mut Vec<u64>) {
        unsafe {
            if START == 0 {
                return;
            }
            let words = (LEN / PAGE).div_ceil(64);
            if out.len() < words {
                out.resize(words, 0);
            }
            for (i, w) in out[..words].iter_mut().enumerate() {
                *w |= *BITS.add(i);
            }
        }
    }

    pub fn active() -> bool {
        unsafe { START != 0 }
    }
    pub fn faults() -> u64 {
        unsafe { FAULTS }
    }
}

#[cfg(unix)]
pub use imp_unix::*;

/// Linux: the host block's base copied in the background, as `bg` on Windows (see there): at capture the range goes
/// read-only (`mprotect`), a helper thread copies it into the base buffer page by page and makes each copied run
/// writable again, and a write that gets to a page first raises SIGSEGV; the shared handler (`imp_unix`) copies that
/// page itself (or waits for the helper's copy of it) and makes it writable before the write re-runs. The capture
/// costs one `mprotect` instead of a 64 MB copy on the game thread.
///
/// The range stays registered on the userfaultfd write watch (`snapshot::wpwatch`) the whole time: `mprotect` changes
/// the VMA's permissions only, a page's write-protect marker stays, so a write after the handler made the page
/// writable still resolves through the watch and reads as written at the next save (soft-dirty likewise; either way
/// a page the helper merely made writable reads as unwritten, as it should). The handler runs inside whatever code
/// faulted: no locks, no allocation (the per-page state is allocated at `begin`, on the system heap).
/// Per page: 0 = not copied, 1 = being copied, 2 = copied (the base buffer holds the page as it was at capture).
/// A page that is not yet 2 was not written since the capture: its live bytes are its base bytes.
#[cfg(target_os = "linux")]
pub mod bg {
    use core::ffi::c_void;
    use core::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering::*};
    const PAGE: usize = 4096;
    /// Pages per helper step (one mprotect each).
    const RUN: usize = 64;

    // Plain atomics, read by the signal handler. They are set at `begin`, before the capture copies the statics, and
    // stay as they are until `release_copy` at the base's drop, so a rollback load (which rewinds the statics) writes
    // back the same values.
    static START: AtomicUsize = AtomicUsize::new(0);
    static LEN: AtomicUsize = AtomicUsize::new(0);
    static DST: AtomicUsize = AtomicUsize::new(0);
    static STATE: AtomicUsize = AtomicUsize::new(0);
    /// The address of the control block's fault counter (set at `begin`; the handler counts there).
    static FAULTS_AT: AtomicUsize = AtomicUsize::new(0);
    /// The last range `begin` protected, kept after `release_copy`: a fault another thread took in it just before the
    /// release made the range writable again re-runs the write instead of being passed on as not ours.
    static LAST_START: AtomicUsize = AtomicUsize::new(0);
    static LAST_LEN: AtomicUsize = AtomicUsize::new(0);

    /// The helper's control block, on the system heap (`SysStatic`): a rollback load rewinds every static of the
    /// executable, so a job flag kept in a static could be written back to "requested" or "running" after the helper
    /// finished, and `release_copy` would wait forever.
    struct Ctl {
        /// 0 idle, 1 a copy requested (`begin`), 2 the helper is copying
        job: AtomicU8,
        /// set by `release_copy`: the helper stops after its current run of pages
        stop: AtomicBool,
        /// set by the helper once it runs
        ready: AtomicBool,
        faults: AtomicUsize,
        /// MELEE_SNAP_BGCOPY_DELAY_MS (test knob): the helper starts each copy this many ms late, so the first frames'
        /// writes (and a synctest's loads) take the fault path
        delay_ms: AtomicU64,
    }
    /// The control block's address (a leaked box made on first use; it never moves).
    static CTL: crate::arena::SysStatic<usize> = crate::arena::SysStatic::new(|| 0);
    /// The helper, to unpark it (only the game thread touches this).
    static HELPER: crate::arena::SysStatic<Option<std::thread::Thread>> = crate::arena::SysStatic::new(|| None);
    /// The per-page state buffer, kept and cleared from one base to the next (system heap).
    static STATE_BUF: crate::arena::SysStatic<Vec<AtomicU8>> = crate::arena::SysStatic::new(Vec::new);

    fn ctl() -> &'static Ctl {
        let p = CTL.with(|p| {
            if *p == 0 {
                let c = Ctl {
                    job: AtomicU8::new(0),
                    stop: AtomicBool::new(false),
                    ready: AtomicBool::new(false),
                    faults: AtomicUsize::new(0),
                    delay_ms: AtomicU64::new(0),
                };
                *p = Box::leak(Box::new(c)) as *const Ctl as usize;
            }
            *p
        });
        unsafe { &*(p as *const Ctl) }
    }

    fn helper() -> Option<std::thread::Thread> {
        HELPER.with(|h| h.clone())
    }

    unsafe fn state(i: usize) -> &'static AtomicU8 {
        &*(STATE.load(Relaxed) as *const AtomicU8).add(i)
    }

    /// Copy page `i` into the base buffer unless someone did (or is doing) it; returns once it is copied.
    unsafe fn claim_copy(i: usize) {
        let st = state(i);
        loop {
            match st.compare_exchange(0, 1, Acquire, Acquire) {
                Ok(_) => {
                    let (s, d) = (START.load(Relaxed), DST.load(Relaxed));
                    core::ptr::copy_nonoverlapping((s + i * PAGE) as *const u8, (d + i * PAGE) as *mut u8, PAGE);
                    st.store(2, Release);
                    return;
                }
                Err(2) => return,
                Err(_) => core::hint::spin_loop(),
            }
        }
    }

    /// The SIGSEGV handler's part: true when `addr` is in the range (the page is now copied and writable, re-run).
    pub(crate) unsafe fn on_fault(addr: usize) -> bool {
        let (s, l) = (START.load(Acquire), LEN.load(Relaxed));
        if s == 0 || addr < s || addr >= s + l {
            let (ls, ll) = (LAST_START.load(Relaxed), LAST_LEN.load(Relaxed));
            // released meanwhile: the whole range is writable again
            return s == 0 && ls != 0 && addr >= ls && addr < ls + ll;
        }
        let i = (addr - s) / PAGE;
        claim_copy(i);
        let f = FAULTS_AT.load(Relaxed);
        if f != 0 {
            (*(f as *const AtomicUsize)).fetch_add(1, Relaxed);
        }
        libc::mprotect((s + i * PAGE) as *mut c_void, PAGE, libc::PROT_READ | libc::PROT_WRITE);
        true
    }

    /// Start the copy helper unless it runs: one thread for the whole process, parked between copies. Called by
    /// `snapshot::prewarm` (before a match's capture) and by `begin` when nothing called that.
    ///
    /// Why one thread that never ends: a thread's start and end write the runtime's statics (std's and glibc's thread
    /// bookkeeping), and a rollback load rewinds every static of the executable on Linux (no PDB names std's own
    /// objects as host-only there). A helper per base that ended mid-session had a later load write its old
    /// bookkeeping back, and the next thread std started at the session's end panicked in a btree assertion (2 of 4
    /// synctests; 0 of 6 with the helper kept alive). This returns once the thread runs, so its start is done before a
    /// capture copies the statics; it never ends.
    pub fn spawn_helper() {
        let c = ctl();
        if HELPER.with(|h| h.is_some()) {
            return;
        }
        let _s = crate::arena::SysScope::enter();
        let h = std::thread::Builder::new().name("snapshot-base-copy".into()).spawn(move || {
            c.ready.store(true, Release);
            loop {
                while c.job.load(Acquire) != 1 {
                    std::thread::park();
                }
                c.job.store(2, Release);
                let delay = c.delay_ms.load(Relaxed);
                if delay > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(delay));
                }
                let (start, pages) = (START.load(Acquire), LEN.load(Relaxed) / PAGE);
                let mut i = 0;
                while start != 0 && i < pages && !c.stop.load(Acquire) {
                    let n = RUN.min(pages - i);
                    for k in i..i + n {
                        unsafe { claim_copy(k) };
                    }
                    // every page of the run is copied: writes to it need no fault any more
                    unsafe { libc::mprotect((start + i * PAGE) as *mut c_void, n * PAGE, libc::PROT_READ | libc::PROT_WRITE) };
                    i += n;
                }
                c.job.store(0, Release);
            }
        });
        if let Ok(h) = h {
            while !c.ready.load(Acquire) {
                std::thread::yield_now();
            }
            // the handle is dropped (detached); the Thread is kept to unpark it
            HELPER.with(|t| *t = Some(h.thread().clone()));
        }
    }

    /// Start: `[start, start+len)` read-only, its base copied into `dst` (len bytes) by the helper.
    pub unsafe fn begin(start: usize, len: usize, dst: *mut u8) -> Result<(), String> {
        if START.load(Relaxed) != 0 {
            return Err("a background base copy is already active".into());
        }
        if start % PAGE != 0 || len % PAGE != 0 || len == 0 {
            return Err("the range is not page aligned".into());
        }
        spawn_helper();
        let c = ctl();
        let pages = len / PAGE;
        let ptr = STATE_BUF.with(|buf| {
            if buf.len() < pages {
                *buf = Vec::from_iter((0..pages).map(|_| AtomicU8::new(0)));
            } else {
                buf[..pages].iter().for_each(|a| a.store(0, Relaxed));
            }
            buf.as_ptr() as usize
        });
        super::imp_unix::install()?;
        c.faults.store(0, Relaxed);
        c.stop.store(false, Relaxed);
        let delay = {
            let _s = crate::arena::SysScope::enter();
            std::env::var("MELEE_SNAP_BGCOPY_DELAY_MS").ok().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0)
        };
        c.delay_ms.store(delay, Relaxed);
        FAULTS_AT.store(&c.faults as *const AtomicUsize as usize, Relaxed);
        STATE.store(ptr, Relaxed);
        DST.store(dst as usize, Relaxed);
        LEN.store(len, Relaxed);
        LAST_LEN.store(len, Relaxed);
        LAST_START.store(start, Relaxed);
        START.store(start, Release);
        if libc::mprotect(start as *mut c_void, len, libc::PROT_READ) != 0 {
            // nothing was made read-only: no page needs a copy
            START.store(0, Release);
            super::imp_unix::uninstall_if_idle();
            return Err(format!("mprotect failed ({})", std::io::Error::last_os_error()));
        }
        // no helper (it could not start): the handler alone copies; pages never written stay read-only until
        // `release_copy`
        if let Some(t) = helper() {
            c.job.store(1, Release);
            t.unpark();
        }
        Ok(())
    }

    /// The base copy of page `i` is in the buffer (else the live page is still the base page).
    #[inline]
    pub fn copied(i: usize) -> bool {
        let s = START.load(Relaxed);
        s != 0 && i < LEN.load(Relaxed) / PAGE && unsafe { state(i).load(Acquire) == 2 }
    }

    pub fn active() -> bool {
        START.load(Relaxed) != 0
    }

    pub fn faults() -> usize {
        ctl().faults.load(Relaxed)
    }

    /// Stop: the helper ends its copy, every page is copied, the range writable. The base buffer is then complete (the
    /// caller keeps using it).
    pub unsafe fn release_copy() {
        let s = START.load(Relaxed);
        if s == 0 {
            return;
        }
        let c = ctl();
        c.stop.store(true, Release);
        if let Some(t) = helper() {
            // a copy not picked up yet is picked up and stopped at once; one in progress ends after its run
            t.unpark();
            while c.job.load(Acquire) != 0 {
                std::thread::yield_now();
            }
        }
        let l = LEN.load(Relaxed);
        for i in 0..l / PAGE {
            claim_copy(i);
        }
        libc::mprotect(s as *mut c_void, l, libc::PROT_READ | libc::PROT_WRITE);
        START.store(0, Release);
        super::imp_unix::uninstall_if_idle();
    }
}
