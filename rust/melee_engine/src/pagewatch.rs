//! Write tracking for the executable's writable sections (the statics), which `GetWriteWatch` cannot watch (image
//! pages). A watched page is read-only; the first write to it raises an access violation that a vectored handler
//! answers by setting the page's bit and making it writable again. The snapshot then knows which static pages were
//! written since it last protected them, instead of comparing the whole 9 MB on every save and load.
//!
//! The handler can run on any thread and inside any code (the allocator, the copy-on-write handler): it takes no
//! locks, allocates nothing and writes no static (the bitmap is on the system heap; the control block it reads is
//! set before the first protection and cleared after the last release).

#[cfg(windows)]
mod imp {
    use core::ffi::c_void;
    use core::sync::atomic::{AtomicU64, Ordering::*};
    const PAGE: usize = 4096;
    const MAXR: usize = 8;

    extern "system" {
        fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
        fn AddVectoredExceptionHandler(first: u32, h: unsafe extern "system" fn(*mut c_void) -> i32) -> *mut c_void;
        fn RemoveVectoredExceptionHandler(h: *mut c_void) -> u32;
    }
    const PAGE_READWRITE: u32 = 0x04;
    const PAGE_READONLY: u32 = 0x02;

    struct Ctl {
        n: usize,
        start: [usize; MAXR],
        len: [usize; MAXR],
        /// Word offset of each range's bits in `bits`.
        off: [usize; MAXR],
        bits: *const AtomicU64,
        handler: *mut c_void,
    }

    static mut CTL: *mut Ctl = core::ptr::null_mut();

    unsafe extern "system" fn handler(info: *mut c_void) -> i32 {
        let rec = *(info as *const *const u8);
        if *(rec as *const u32) != 0xC000_0005 || *(rec.add(24) as *const u32) < 2 {
            return 0;
        }
        let c = CTL;
        if c.is_null() || *(rec.add(32) as *const usize) != 1 {
            return 0;
        }
        let addr = *(rec.add(40) as *const usize);
        let c = &*c;
        for k in 0..c.n {
            if addr >= c.start[k] && addr < c.start[k] + c.len[k] {
                let idx = (addr - c.start[k]) / PAGE;
                (*c.bits.add(c.off[k] + idx / 64)).fetch_or(1 << (idx % 64), SeqCst);
                let mut old = 0u32;
                if VirtualProtect((addr & !(PAGE - 1)) as *mut c_void, PAGE, PAGE_READWRITE, &mut old) == 0 {
                    return 0;
                }
                return -1; // EXCEPTION_CONTINUE_EXECUTION
            }
        }
        0
    }

    /// Start watching `ranges` (page aligned): every page read-only, every first write recorded. One set at a time.
    pub unsafe fn watch(ranges: &[(usize, usize)]) -> Result<(), String> {
        if !CTL.is_null() {
            return Err("a static write watch is already active".into());
        }
        if ranges.len() > MAXR {
            return Err("too many static ranges".into());
        }
        let mut c = Ctl { n: ranges.len(), start: [0; MAXR], len: [0; MAXR], off: [0; MAXR], bits: core::ptr::null(), handler: core::ptr::null_mut() };
        let mut words = 0;
        for (k, &(a, l)) in ranges.iter().enumerate() {
            c.start[k] = a;
            c.len[k] = l;
            c.off[k] = words;
            words += (l / PAGE).div_ceil(64);
        }
        let bits: &'static mut [AtomicU64] = (0..words).map(|_| AtomicU64::new(0)).collect::<Vec<_>>().leak();
        c.bits = bits.as_ptr();
        let c = Box::into_raw(Box::new(c));
        (*c).handler = AddVectoredExceptionHandler(1, handler);
        CTL = c;
        for &(a, l) in ranges {
            let mut old = 0u32;
            if VirtualProtect(a as *mut c_void, l, PAGE_READONLY, &mut old) == 0 {
                release();
                return Err("VirtualProtect failed on a static section".into());
            }
        }
        Ok(())
    }

    /// Stop watching: every page writable again.
    pub unsafe fn release() {
        let c = CTL;
        if c.is_null() {
            return;
        }
        for k in 0..(*c).n {
            let mut old = 0u32;
            VirtualProtect((*c).start[k] as *mut c_void, (*c).len[k], PAGE_READWRITE, &mut old);
        }
        // the store below may itself fault (CTL is a static): the handler still sees the old control block then
        CTL = core::ptr::null_mut();
        RemoveVectoredExceptionHandler((*c).handler);
    }

    pub fn active() -> bool {
        unsafe { !CTL.is_null() }
    }

    /// OR the pages of range `k` written since the previous call into `out` (and clear them).
    pub fn take(k: usize, out: &mut Vec<u64>) {
        unsafe {
            let c = CTL;
            if c.is_null() || k >= (*c).n {
                return;
            }
            let words = ((*c).len[k] / PAGE).div_ceil(64);
            if out.len() < words {
                out.resize(words, 0);
            }
            for (i, w) in out[..words].iter_mut().enumerate() {
                *w |= (*(*c).bits.add((*c).off[k] + i)).swap(0, SeqCst);
            }
        }
    }

    /// Make pages `pages` (ascending indexes) of range `k` read-only again; runs of adjacent pages take one call.
    pub unsafe fn protect(k: usize, pages: &[u32]) {
        let c = CTL;
        if c.is_null() || k >= (*c).n {
            return;
        }
        let mut i = 0;
        while i < pages.len() {
            let s = pages[i];
            let mut e = s + 1;
            i += 1;
            while i < pages.len() && pages[i] == e {
                e += 1;
                i += 1;
            }
            let mut old = 0u32;
            VirtualProtect(((*c).start[k] + s as usize * PAGE) as *mut c_void, (e - s) as usize * PAGE, PAGE_READONLY, &mut old);
        }
    }
}

#[cfg(windows)]
pub use imp::*;

#[cfg(not(windows))]
mod imp {
    pub unsafe fn watch(_: &[(usize, usize)]) -> Result<(), String> {
        Err("no static write watch on this platform".into())
    }
    pub unsafe fn release() {}
    pub fn active() -> bool {
        false
    }
    pub fn take(_: usize, _: &mut Vec<u64>) {}
    pub unsafe fn protect(_: usize, _: &[u32]) {}
}
#[cfg(not(windows))]
pub use imp::*;
