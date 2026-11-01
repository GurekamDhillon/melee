//! Memory below 4 GiB. The C ports keep `u32` guest addresses (relocation words, `Locate`, heap cells), so
//! the arenas the raw archive images and the OS heaps live in must have 32-bit addresses.
//! Reserved with `VirtualAlloc` (Windows) or `mmap` (unix); never freed (an arena lives as long as the process).

use core::ffi::c_void;

/// A zeroed, read-write block whose whole range is below 4 GiB.
#[derive(Clone, Copy, Debug)]
pub struct LowBlock {
    pub base: *mut u8,
    pub len: usize,
}
unsafe impl Send for LowBlock {}
unsafe impl Sync for LowBlock {}

impl LowBlock {
    pub fn addr32(&self) -> u32 {
        self.base as usize as u32
    }
    pub fn end_ptr(&self) -> *mut u8 {
        self.base.wrapping_add(self.len)
    }
}

#[cfg(windows)]
mod sys {
    use super::*;
    extern "system" {
        fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
    }
    const MEM_COMMIT: u32 = 0x1000;
    const MEM_RESERVE: u32 = 0x2000;
    /// Lets a rollback ask which pages were written (`GetWriteWatch`).
    const MEM_WRITE_WATCH: u32 = 0x200000;
    const PAGE_READWRITE: u32 = 0x04;
    pub fn alloc_low(len: usize) -> Option<*mut u8> {
        // A 64-bit process has no low addresses by default: probe 16 MiB-stepped hints from 2 GiB + 16 MiB up to
        // 4 GiB. Guest main-RAM addresses are 0x80000000 and up (the C tells ARAM from MEM1 by that bit).
        let mut hint: usize = 0x8100_0000;
        while hint + len <= 0x1_0000_0000 {
            let p = unsafe { VirtualAlloc(hint as *mut c_void, len, MEM_RESERVE | MEM_COMMIT | MEM_WRITE_WATCH, PAGE_READWRITE) };
            if !p.is_null() {
                return Some(p as *mut u8);
            }
            hint += 0x0100_0000;
        }
        None
    }
}

#[cfg(unix)]
mod sys {
    use super::*;
    extern "C" {
        fn mmap(addr: *mut c_void, len: usize, prot: i32, flags: i32, fd: i32, off: i64) -> *mut c_void;
    }
    pub fn alloc_low(len: usize) -> Option<*mut u8> {
        const PROT_RW: i32 = 3;
        #[cfg(target_os = "linux")]
        const FLAGS: i32 = 0x02 | 0x20 | 0x40; // MAP_PRIVATE | MAP_ANONYMOUS | MAP_32BIT
        #[cfg(not(target_os = "linux"))]
        const FLAGS: i32 = 0x0002 | 0x1000; // MAP_PRIVATE | MAP_ANON (macOS/BSD: hint-probed below)
        let mut hint: usize = 0x8100_0000;
        loop {
            let p = unsafe { mmap(hint as *mut c_void, len, PROT_RW, FLAGS, -1, 0) };
            if p as isize != -1 && (p as usize) + len <= 0x1_0000_0000 {
                return Some(p as *mut u8);
            }
            hint += 0x0100_0000;
            if hint + len > 0x1_0000_0000 {
                return None;
            }
        }
    }
}

/// Reserve and commit `len` zeroed bytes below 4 GiB.
pub fn alloc_low(len: usize) -> Result<LowBlock, String> {
    let len = (len + 0xFFFF) & !0xFFFF;
    match sys::alloc_low(len) {
        Some(p) if (p as usize) + len <= 0x1_0000_0000 => Ok(LowBlock { base: p, len }),
        _ => Err(format!("no {len:#x}-byte block below 4 GiB")),
    }
}
