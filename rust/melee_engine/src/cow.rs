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
        let pages = len / PAGE;
        let bits = vec![0u64; pages.div_ceil(64)].leak();
        let store = VirtualAlloc(core::ptr::null_mut(), len, MEM_RESERVE, PAGE_READWRITE);
        if store.is_null() {
            return Err("cannot reserve the copy-on-write store".into());
        }
        STORE = store as usize;
        BITS = bits.as_mut_ptr();
        START = start;
        LEN = len;
        HANDLER = AddVectoredExceptionHandler(1, handler);
        let mut old = 0u32;
        if VirtualProtect(start as *mut c_void, len, PAGE_READONLY, &mut old) == 0 {
            return Err("VirtualProtect failed".into());
        }
        Ok(())
    }

    /// Undo `protect` (the range becomes writable; the store stays reserved).
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

    pub fn active() -> bool {
        unsafe { START != 0 }
    }
    pub fn faults() -> u64 {
        unsafe { FAULTS }
    }
}

#[cfg(windows)]
pub use imp::*;
