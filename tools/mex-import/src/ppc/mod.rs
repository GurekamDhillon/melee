//! PPC32 (Gekko) -> IR decoder and interpreter for m-ex mod code (platform night 3 step 3).
//!
//! `ir` decodes exactly the subset pc/platform/gw_ppc.c executes; `interp` runs it op for op as
//! that C does (checked against the C itself: oracle/ppc_oracle.sh, tests/ppc_oracle.rs).
//! Memory goes through [`interp::Mem`]: [`FlatMem`] is plain big-endian guest bytes,
//! `crate::remap::GuestSpace` rewrites accesses into native little-endian structs through the
//! GameCube offset tables.
pub mod interp;
pub mod ir;

pub use interp::{Bridge, Cpu, Fault, Machine, Mem, Step};

/// Plain big-endian guest memory at `[base, base + bytes.len())`.
pub struct FlatMem {
    pub base: u32,
    pub bytes: Vec<u8>,
}

impl FlatMem {
    fn off(&self, ea: u32, n: u32) -> Result<usize, Fault> {
        if ea < self.base || ea as u64 + n as u64 > self.base as u64 + self.bytes.len() as u64 {
            return Err(Fault::Mem(ea));
        }
        Ok((ea - self.base) as usize)
    }
    fn ld<const N: usize>(&self, ea: u32) -> Result<[u8; N], Fault> {
        let o = self.off(ea, N as u32)?;
        Ok(self.bytes[o..o + N].try_into().unwrap())
    }
    fn st(&mut self, ea: u32, b: &[u8]) -> Result<(), Fault> {
        let o = self.off(ea, b.len() as u32)?;
        self.bytes[o..o + b.len()].copy_from_slice(b);
        Ok(())
    }
}

impl Mem for FlatMem {
    fn ld8(&mut self, ea: u32) -> Result<u8, Fault> {
        Ok(self.ld::<1>(ea)?[0])
    }
    fn ld16(&mut self, ea: u32) -> Result<u16, Fault> {
        Ok(u16::from_be_bytes(self.ld(ea)?))
    }
    fn ld32(&mut self, ea: u32) -> Result<u32, Fault> {
        Ok(u32::from_be_bytes(self.ld(ea)?))
    }
    fn ld64(&mut self, ea: u32) -> Result<u64, Fault> {
        Ok(u64::from_be_bytes(self.ld(ea)?))
    }
    fn st8(&mut self, ea: u32, v: u8) -> Result<(), Fault> {
        self.st(ea, &[v])
    }
    fn st16(&mut self, ea: u32, v: u16) -> Result<(), Fault> {
        self.st(ea, &v.to_be_bytes())
    }
    fn st32(&mut self, ea: u32, v: u32) -> Result<(), Fault> {
        self.st(ea, &v.to_be_bytes())
    }
    fn st64(&mut self, ea: u32, v: u64) -> Result<(), Fault> {
        self.st(ea, &v.to_be_bytes())
    }
}
