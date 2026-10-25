//! A C `va_list` for the ports: the variadic arguments of one call as a slice of machine words
//! (pointers as they are, integers sign-extended, a float widened to double and passed by its bits),
//! stepped like the shim behind the game's `__va_arg` (`src/MSL/stdarg.h`, `pc/platform/shim_libc.c`).
use core::ffi::c_void;

pub struct Va {
    args: *const *const c_void,
    n: usize,
    i: usize,
    slot: u64,
}

impl Va {
    /// the list of one call; `args` must outlive every use of the `Va`
    pub fn new(args: &[*const c_void]) -> Va {
        Va { args: args.as_ptr(), n: args.len(), i: 0, slot: 0 }
    }

    /// `__va_arg(list, info)`: the low 16 bits of `info` are the size of the requested type, bit 16 marks
    /// float / double (the caller passed a double: an `f32` request narrows it). Returns a pointer to a copy of
    /// the argument that lives until the next call. An exhausted list reads zero.
    pub unsafe fn arg(&mut self, info: u32) -> *mut c_void {
        let w = if self.i < self.n { *self.args.add(self.i) as usize as u64 } else { 0 };
        self.i += 1;
        let size = (info & 0xFFFF) as usize;
        let mut slot = [0u8; 8];
        if info & 0x10000 != 0 {
            let d = f64::from_bits(w);
            if size == 4 {
                slot[..4].copy_from_slice(&(d as f32).to_le_bytes());
            } else {
                slot = d.to_le_bytes();
            }
        } else {
            let b = w.to_le_bytes();
            let n = size.min(8);
            slot[..n].copy_from_slice(&b[..n]);
        }
        self.slot = u64::from_le_bytes(slot);
        &mut self.slot as *mut u64 as *mut c_void
    }
}
