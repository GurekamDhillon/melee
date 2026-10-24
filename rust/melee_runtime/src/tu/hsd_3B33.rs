//! Port of src/sysdolphin/baselib/hsd_3B33.c: the JPEG encoder's output sink. `hsd_803B3344`
//! appends one byte and `hsd_803B3398` a block to the output buffer; when it is full the C
//! `longjmp`s to `hsd_804D2648.buf` (set up by hsd_803B3408 in hsd_3B34.c, unported). The native
//! equivalent of that non-local exit is an `Err(JpegOverflow)` the caller propagates to the
//! point where it called `setjmp`.
//!
//! The three globals the sink uses (`hsd_804D79A0` write cursor, `hsd_804D79A4` buffer start,
//! `hsd_804D79A8` buffer size) are defined in hsd_3B34.c and live in [`Hsd3B33`] until that TU is
//! ported.
#![allow(non_snake_case)]

/// The `longjmp(hsd_804D2648.buf, true)` of the C: the output buffer is full.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JpegOverflow;

/// `hsd_804D79A0`, `hsd_804D79A4`, `hsd_804D79A8`.
#[derive(Clone, Copy, Debug)]
pub struct Hsd3B33 {
    /// `hsd_804D79A0`: write cursor.
    pub cur: *mut u8,
    /// `hsd_804D79A4`: buffer start.
    pub base: *mut u8,
    /// `hsd_804D79A8`: buffer size.
    pub size: i32,
}

impl Default for Hsd3B33 {
    fn default() -> Self {
        Hsd3B33 { cur: core::ptr::null_mut(), base: core::ptr::null_mut(), size: 0 }
    }
}

impl Hsd3B33 {
    fn end(&self) -> usize {
        (self.base as usize).wrapping_add(self.size as u32 as usize)
    }

    /// `hsd_803B3344` (0x803B3344): append a byte.
    ///
    /// # Safety
    /// `cur`/`base`/`size` must describe a writable buffer.
    pub unsafe fn hsd_803B3344(&mut self, byte: u8) -> Result<(), JpegOverflow> {
        let p = self.cur;
        if (p as usize) < self.end() {
            self.cur = p.add(1);
            *p = byte;
            return Ok(());
        }
        Err(JpegOverflow)
    }

    /// `hsd_803B3398` (0x803B3398): append `size` bytes from `src`.
    ///
    /// # Safety
    /// `src` must be readable for `size` bytes; the buffer as for `hsd_803B3344`.
    pub unsafe fn hsd_803B3398(&mut self, src: *const u8, size: usize) -> Result<(), JpegOverflow> {
        let p = self.cur;
        if (p as usize) < self.end().wrapping_sub(size) {
            core::ptr::copy_nonoverlapping(src, p, size);
            self.cur = p.add(size);
            return Ok(());
        }
        Err(JpegOverflow)
    }
}
