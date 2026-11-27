//! The IR interpreter: pc/platform/gw_ppc.c's execute core, op for op, over a [`Mem`] (where the
//! field rewriting happens, `crate::remap`) and a [`Bridge`] (calls that leave the blob).
//!
//! Integer arithmetic that is signed-overflow UB in the C (`mulli`, `mullw`) wraps, which is
//! what clang -O2 emits for it on i686; the float-to-int conversions (`fctiwz`, `fctiw`) return
//! 0x80000000 for NaN and out-of-range values, as cvttsd2si does. Both are checked against the C
//! itself by tests/ppc_oracle.rs.

use super::ir::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Load/store the memory refused (address).
    Mem(u32),
    /// gw_ppc_bad_opcode (instruction address, word).
    BadOpcode(u32, u32),
    /// gw_panic: an unmodelled SPR (instruction address).
    Panic(u32),
    /// Fetch outside every code range.
    Fetch(u32),
    /// Instruction budget exhausted.
    Budget,
    /// The bridge could not resolve a native target.
    Unresolved(u32),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cpu {
    pub gpr: [u32; 32],
    /// Raw 64-bit FPR contents (the C's `gw_ppc_fpr.u64`; `.d` is f64::from_bits).
    pub fpr: [u64; 32],
    pub lr: u32,
    pub ctr: u32,
    pub xer: u32,
    pub cr: u32,
    pub pc: u32,
}

impl Cpu {
    pub fn f(&self, i: u8) -> f64 {
        f64::from_bits(self.fpr[i as usize])
    }
    pub fn set_f(&mut self, i: u8, v: f64) {
        self.fpr[i as usize] = v.to_bits();
    }
    fn r(&self, i: u8) -> u32 {
        self.gpr[i as usize]
    }
    fn r0(&self, i: u8) -> u32 {
        if i == 0 { 0 } else { self.gpr[i as usize] }
    }
    fn cr_bit(&self, bi: u8) -> u32 {
        (self.cr >> (31 - bi as u32)) & 1
    }
    fn cr_set_bit(&mut self, bi: u8, v: u32) {
        let m = 1u32 << (31 - bi as u32);
        self.cr = (self.cr & !m) | ((v & 1) << (31 - bi as u32));
    }
    fn cr_set_field(&mut self, crf: u8, field: u32) {
        let sh = 28 - 4 * crf as u32;
        self.cr = (self.cr & !(0xF << sh)) | ((field & 0xF) << sh);
    }
    fn cr_get_field(&self, crf: u8) -> u32 {
        (self.cr >> (28 - 4 * crf as u32)) & 0xF
    }
    fn ca(&self) -> u32 {
        (self.xer >> 29) & 1
    }
    fn set_ca(&mut self, v: bool) {
        self.xer = (self.xer & !0x2000_0000) | ((v as u32) << 29);
    }
    fn cr0(&mut self, res: u32) {
        let lt = res >> 31;
        let gt = (res != 0 && lt == 0) as u32;
        let eq = (res == 0) as u32;
        let so = self.xer >> 31;
        self.cr_set_field(0, lt << 3 | gt << 2 | eq << 1 | so);
    }
    fn cmp(&mut self, crf: u8, a: u32, b: u32, unsigned: bool) {
        let (lt, gt) = if unsigned { (a < b, a > b) } else { ((a as i32) < (b as i32), (a as i32) > (b as i32)) };
        let so = self.xer >> 31;
        self.cr_set_field(crf, (lt as u32) << 3 | (gt as u32) << 2 | ((a == b) as u32) << 1 | so);
    }
    /// gw_ppc_cond (decrements CTR for the CTR forms).
    fn cond(&mut self, bo: u8, bi: u8) -> bool {
        let crbit = self.cr_bit(bi);
        let bo = bo as u32;
        if bo & 0x10 != 0 {
            if bo & 0x04 != 0 {
                return true;
            }
            self.ctr = self.ctr.wrapping_sub(1);
            return (self.ctr != 0) ^ (bo & 0x02 != 0);
        }
        let crok = (crbit ^ ((bo & 0x08 == 0) as u32)) != 0;
        if bo & 0x04 != 0 {
            return crok;
        }
        self.ctr = self.ctr.wrapping_sub(1);
        ((self.ctr != 0) ^ (bo & 0x02 != 0)) & crok
    }
}

/// Guest memory as the interpreter sees it: big-endian values at guest addresses.
pub trait Mem {
    fn ld8(&mut self, ea: u32) -> Result<u8, Fault>;
    fn ld16(&mut self, ea: u32) -> Result<u16, Fault>;
    fn ld32(&mut self, ea: u32) -> Result<u32, Fault>;
    fn ld64(&mut self, ea: u32) -> Result<u64, Fault>;
    fn st8(&mut self, ea: u32, v: u8) -> Result<(), Fault>;
    fn st16(&mut self, ea: u32, v: u16) -> Result<(), Fault>;
    fn st32(&mut self, ea: u32, v: u32) -> Result<(), Fault>;
    fn st64(&mut self, ea: u32, v: u64) -> Result<(), Fault>;
    /// Instruction fetch (code is plain big-endian guest bytes).
    fn fetch(&mut self, ea: u32) -> Result<u32, Fault> {
        self.ld32(ea)
    }
}

/// Everything outside the blob: what gw_ppc_bridge_call reaches through the resolver.
pub trait Bridge<M: Mem> {
    /// Is `addr` interpreted guest code (gw_ppc_in_blob)?
    fn in_blob(&self, addr: u32) -> bool;
    /// Call the native target (args r3.., f1..; result into r3 / f1).
    fn call(&mut self, cpu: &mut Cpu, mem: &mut M, target: u32) -> Result<(), Fault>;
}

/// Outcome of one instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Continue,
    /// Returned to the native caller (r3 holds the result).
    Return,
}

fn ctiw(b: f64) -> u64 {
    // cvttsd2si: the integer indefinite 0x80000000 for NaN / out of range.
    let iv = if b > -2147483649.0 && b < 2147483648.0 { b as i32 } else { i32::MIN };
    iv as i64 as u64
}

fn rotl(v: u32, sh: u32) -> u32 {
    v.rotate_left(sh & 31)
}

/// Gekko's own float-to-int: toward zero (fctiwz) or to nearest even (fctiw; FPSCR RN = 0, as the
/// game leaves it), saturating to 0x7FFFFFFF / 0x80000000, NaN giving 0x80000000. The high word is
/// 0xFFF80000 as on hardware (the game only stores the low word, stfd + lwz).
fn ctiw_gekko(b: f64, nearest: bool) -> u64 {
    let r = if nearest { b.round_ties_even() } else { b.trunc() };
    let iv = if b.is_nan() {
        i32::MIN
    } else if r >= 2147483647.0 {
        i32::MAX
    } else if r <= -2147483648.0 {
        i32::MIN
    } else {
        r as i32
    };
    // a zero result of a negative operand (-0, -0.3 toward zero) sets bit 32 (Dolphin, from hardware tests)
    let neg_zero = if iv == 0 && b.is_sign_negative() { 0x1_0000_0000 } else { 0 };
    0xFFF8_0000_0000_0000 | neg_zero | (iv as u32 as u64)
}

/// Dolphin's interpreter float semantics (Interpreter_FPUtils.h, Interpreter_FloatingPoint.cpp), used in Gekko
/// mode; checked against an independent model by tests/gekko_trust.rs (notes/referee.md section 1).
mod dol {
    const EXP: u64 = 0x7FF0_0000_0000_0000;
    const FRAC: u64 = 0x000F_FFFF_FFFF_FFFF;
    const QBIT: u64 = 0x0008_0000_0000_0000;
    pub const SIGN: u64 = 1 << 63;
    /// The default NaN of an invalid operation: positive, payload 0 (the x86 host's default NaN is negative).
    pub const PPC_NAN: u64 = 0x7FF8_0000_0000_0000;
    pub fn is_nan(x: u64) -> bool {
        x & EXP == EXP && x & FRAC != 0
    }
    /// NI_add / NI_sub / NI_mul / NI_div: a NaN result is frA's NaN (quieted), else frB's, else the default NaN.
    pub fn ni2(a: u64, b: u64, r: f64) -> u64 {
        if r.is_nan() {
            if is_nan(a) {
                return a | QBIT;
            }
            if is_nan(b) {
                return b | QBIT;
            }
            return PPC_NAN;
        }
        r.to_bits()
    }
    /// NI_madd / NI_msub: fma(a, c, +-b), NaN operands checked in the order a, b, c (b as given, not negated).
    pub fn ni_fma(a: u64, c: u64, b: u64, sub: bool) -> u64 {
        for x in [a, b, c] {
            if is_nan(x) {
                return x | QBIT;
            }
        }
        let bb = if sub { -f64::from_bits(b) } else { f64::from_bits(b) };
        let r = f64::from_bits(a).mul_add(f64::from_bits(c), bb);
        if r.is_nan() { PPC_NAN } else { r.to_bits() }
    }
    /// Force25Bit: frC of a single multiply keeps 25 mantissa bits, rounded; a denormal is normalised first.
    pub fn force25(x: u64) -> u64 {
        let (e, f) = (x & EXP, x & FRAC);
        if e == 0 && f != 0 {
            let shift = f.leading_zeros() - 11;
            let keep = ((0xFFFF_FFFF_F800_0000u64 as i64) >> shift) as u64;
            (x & keep).wrapping_add(x & (0x800_0000u64 >> shift))
        } else {
            (x & 0xFFFF_FFFF_F800_0000).wrapping_add(x & 0x800_0000)
        }
    }
    /// stfs (ConvertToSingle): truncating; a double in the single denormal range is shifted, not rounded.
    pub fn to_single(x: u64) -> u32 {
        let e = ((x >> 52) & 0x7FF) as u32;
        if (874..=896).contains(&e) && x & !SIGN != 0 {
            let t = (0x8000_0000u32 | ((x & FRAC) >> 21) as u32) >> (905 - e);
            t | ((x >> 32) & 0x8000_0000) as u32
        } else {
            (((x >> 32) & 0xC000_0000) | ((x >> 29) & 0x3FFF_FFFF)) as u32
        }
    }
    /// lfs (ConvertToDouble): exact; a signalling NaN stays signalling.
    pub fn from_single(v: u32) -> u64 {
        let x = v as u64;
        let (e, f) = ((x >> 23) & 0xFF, x & 0x7F_FFFF);
        if e == 0 && f != 0 {
            (f32::from_bits(v) as f64).to_bits() // a denormal single is a normal double: exact either way
        } else {
            let y = if e > 0 && e < 255 { ((e >> 7) ^ 1) & 1 } else { e >> 7 };
            ((x & 0xC000_0000) << 32) | (y << 61) | (y << 60) | (y << 59) | ((x & 0x3FFF_FFFF) << 29)
        }
    }
}

pub struct Machine {
    pub cpu: Cpu,
    /// PowerPC-exact floats (the engine's guest runs): fused multiply-add (one rounding, then to
    /// single for the `s` forms) and Gekko's saturating fctiw / fctiwz. Off, the machine is
    /// gw_ppc.c op for op (what tests/ppc_oracle.rs checks).
    pub gekko: bool,
}

impl Machine {
    pub fn new(cpu: Cpu) -> Machine {
        Machine { cpu, gekko: false }
    }
    pub fn new_gekko(cpu: Cpu) -> Machine {
        Machine { cpu, gekko: true }
    }

    fn ea(&self, a: u8, ea: Ea, update: bool) -> u32 {
        let base = if update { self.cpu.r(a) } else { self.cpu.r0(a) };
        match ea {
            Ea::Disp(d) => base.wrapping_add(d as u32),
            Ea::Idx(b) => base.wrapping_add(self.cpu.r(b)),
        }
    }

    /// gw_ppc_bridge_tail.
    fn tail<M: Mem, B: Bridge<M>>(&mut self, mem: &mut M, br: &mut B, target: u32) -> Result<Step, Fault> {
        br.call(&mut self.cpu, mem, target)?;
        if self.cpu.lr == 0 {
            return Ok(Step::Return);
        }
        if br.in_blob(self.cpu.lr) {
            self.cpu.pc = self.cpu.lr;
            return Ok(Step::Continue);
        }
        Ok(Step::Return)
    }

    fn branch<M: Mem, B: Bridge<M>>(&mut self, mem: &mut M, br: &mut B, target: u32, lk: bool) -> Result<Step, Fault> {
        if lk {
            self.cpu.lr = self.cpu.pc;
        }
        if br.in_blob(target) {
            self.cpu.pc = target;
            Ok(Step::Continue)
        } else if lk {
            br.call(&mut self.cpu, mem, target)?;
            Ok(Step::Continue)
        } else {
            self.tail(mem, br, target)
        }
    }

    /// Fetch, decode and execute one instruction at `cpu.pc` (pc advances first, as in C).
    pub fn step<M: Mem, B: Bridge<M>>(&mut self, mem: &mut M, br: &mut B) -> Result<Step, Fault> {
        let ip = self.cpu.pc;
        if !br.in_blob(ip) {
            return Err(Fault::Fetch(ip));
        }
        let w = mem.fetch(ip)?;
        self.cpu.pc = ip.wrapping_add(4);
        self.exec(mem, br, decode(w, ip), ip, w)
    }

    /// Run until the guest returns to the native caller or faults.
    pub fn run<M: Mem, B: Bridge<M>>(&mut self, mem: &mut M, br: &mut B, mut budget: u32) -> Result<u32, Fault> {
        loop {
            if budget == 0 {
                return Err(Fault::Budget);
            }
            budget -= 1;
            if self.step(mem, br)? == Step::Return {
                return Ok(self.cpu.gpr[3]);
            }
        }
    }

    pub fn exec<M: Mem, B: Bridge<M>>(&mut self, mem: &mut M, br: &mut B, op: Op, ip: u32, w: u32) -> Result<Step, Fault> {
        let gekko = self.gekko;
        let c = &mut self.cpu;
        match op {
            Op::AddI { d, a, imm } => c.gpr[d as usize] = c.r0(a).wrapping_add(imm),
            Op::MulI { d, a, imm } => c.gpr[d as usize] = (c.r(a) as i32).wrapping_mul(imm) as u32,
            Op::SubfIc { d, a, imm } => {
                // subfic / addic read rA even when it is r0 (gw_ppc.c reads 0 there)
                let b = if gekko { c.r(a) } else { c.r0(a) };
                c.set_ca(imm >= b);
                c.gpr[d as usize] = imm.wrapping_sub(b);
            }
            Op::AddIc { d, a, imm, rc } => {
                let b = if gekko { c.r(a) } else { c.r0(a) };
                // addic. is primary opcode 13; gw_ppc.c (and the decoder, which mirrors it) takes Rc from bit 0 of
                // the immediate, so `addic` with an odd immediate wrote CR0 and `addic.` with an even one did not
                let rc = if gekko { w >> 26 == 13 } else { rc };
                let (sum, carry) = b.overflowing_add(imm);
                c.set_ca(carry);
                c.gpr[d as usize] = sum;
                if rc {
                    c.cr0(sum);
                }
            }
            Op::OrI { a, s, imm } => c.gpr[a as usize] = c.r(s) | imm,
            Op::XorI { a, s, imm } => c.gpr[a as usize] = c.r(s) ^ imm,
            Op::AndI { a, s, imm } => {
                let v = c.r(s) & imm;
                c.gpr[a as usize] = v;
                c.cr0(v);
            }
            Op::Rlwinm { a, s, sh, mask, rc } => {
                let v = rotl(c.r(s), sh as u32) & mask;
                c.gpr[a as usize] = v;
                if rc {
                    c.cr0(v);
                }
            }
            Op::Rlwimi { a, s, sh, mask, rc } => {
                let v = (rotl(c.r(s), sh as u32) & mask) | (c.r(a) & !mask);
                c.gpr[a as usize] = v;
                if rc {
                    c.cr0(v);
                }
            }
            Op::Rlwnm { a, s, b, mask, rc } => {
                let v = rotl(c.r(s), c.r(b)) & mask;
                c.gpr[a as usize] = v;
                if rc {
                    c.cr0(v);
                }
            }
            Op::CmpI { crf, a, imm, unsigned } => {
                let av = c.r(a);
                c.cmp(crf, av, imm, unsigned)
            }
            Op::Cmp { crf, a, b, unsigned } => {
                let (av, bv) = (c.r(a), c.r(b));
                c.cmp(crf, av, bv, unsigned)
            }
            Op::Arith { op, d, a, b, rc } => {
                let (av, bv, ca) = (c.r(a), c.r(b), c.ca());
                let v = match op {
                    ArithOp::Add => av.wrapping_add(bv),
                    ArithOp::Subf => bv.wrapping_sub(av),
                    ArithOp::Neg => 0u32.wrapping_sub(av),
                    ArithOp::Mullw => (av as i32).wrapping_mul(bv as i32) as u32,
                    ArithOp::Mulhw => ((av as i32 as i64 * bv as i32 as i64) >> 32) as u32,
                    ArithOp::Mulhwu => ((av as u64 * bv as u64) >> 32) as u32,
                    ArithOp::Divw if gekko && (bv == 0 || (av == 0x8000_0000 && bv == 0xFFFF_FFFF)) => {
                        // the undefined results as the hardware gives them: -1 for a negative dividend, else 0
                        if (av as i32) < 0 { 0xFFFF_FFFF } else { 0 }
                    }
                    ArithOp::Divw => {
                        if bv == 0 {
                            0
                        } else if av == 0x8000_0000 && bv == 0xFFFF_FFFF {
                            0x8000_0000
                        } else {
                            ((av as i32) / (bv as i32)) as u32
                        }
                    }
                    ArithOp::Divwu => {
                        if bv == 0 { 0 } else { av / bv }
                    }
                    ArithOp::Subfc => {
                        c.set_ca(bv >= av);
                        bv.wrapping_sub(av)
                    }
                    ArithOp::Subfe => {
                        let s = (!av) as u64 + bv as u64 + ca as u64;
                        c.set_ca(s >= 1 << 32);
                        s as u32
                    }
                    ArithOp::Subfze => {
                        let s = (!av) as u64 + ca as u64;
                        c.set_ca(s >= 1 << 32);
                        s as u32
                    }
                    ArithOp::Subfme => {
                        let s = (!av) as u64 + ca as u64;
                        c.set_ca(s >= 1);
                        (!av).wrapping_add(ca).wrapping_sub(1)
                    }
                    ArithOp::Addze => {
                        let s = av as u64 + ca as u64;
                        c.set_ca(s >= 1 << 32);
                        s as u32
                    }
                    ArithOp::Addc => {
                        let s = av as u64 + bv as u64;
                        c.set_ca(s >= 1 << 32);
                        s as u32
                    }
                    ArithOp::Adde => {
                        let s = av as u64 + bv as u64 + ca as u64;
                        c.set_ca(s >= 1 << 32);
                        s as u32
                    }
                    ArithOp::Addme => {
                        let s = av as u64 + ca as u64;
                        c.set_ca(s >= 1);
                        av.wrapping_add(ca).wrapping_sub(1)
                    }
                };
                c.gpr[d as usize] = v;
                if rc {
                    c.cr0(v);
                }
            }
            Op::Logic { op, a, s, b, rc } => {
                let (sv, bv) = (c.r(s), c.r(b));
                let v = match op {
                    LogicOp::And => sv & bv,
                    LogicOp::Andc => sv & !bv,
                    LogicOp::Or => sv | bv,
                    LogicOp::Orc => sv | !bv,
                    LogicOp::Xor => sv ^ bv,
                    LogicOp::Nor => !(sv | bv),
                    LogicOp::Nand => !(sv & bv),
                    LogicOp::Eqv => !(sv ^ bv),
                    // bit 5 of the amount shifts everything out (gw_ppc.c masks the amount to 5 bits)
                    LogicOp::Slw | LogicOp::Srw if gekko && bv & 0x20 != 0 => 0,
                    LogicOp::Slw => sv << (bv & 31),
                    LogicOp::Srw => sv >> (bv & 31),
                    LogicOp::Sraw => {
                        let sh = bv & 0x3F;
                        let (res, ca) = if sh == 0 {
                            (sv, false)
                        } else if sh >= 32 {
                            (if sv & 0x8000_0000 != 0 { 0xFFFF_FFFF } else { 0 }, sv & 0x8000_0000 != 0)
                        } else {
                            (((sv as i32) >> sh) as u32, sv & 0x8000_0000 != 0 && sv & ((1 << sh) - 1) != 0)
                        };
                        c.set_ca(ca);
                        res
                    }
                };
                c.gpr[a as usize] = v;
                if rc {
                    c.cr0(v);
                }
            }
            Op::Srawi { a, s, sh, rc } => {
                let sv = c.r(s);
                let sh = sh as u32;
                let (res, ca) = if sh == 0 {
                    (sv, false)
                } else {
                    (((sv as i32) >> sh) as u32, sv & 0x8000_0000 != 0 && sv & ((1 << sh) - 1) != 0)
                };
                c.gpr[a as usize] = res;
                c.set_ca(ca);
                if rc {
                    c.cr0(res);
                }
            }
            Op::Un { op, a, s, rc } => {
                let sv = c.r(s);
                let v = match op {
                    UnOp::Cntlzw => sv.leading_zeros(),
                    UnOp::Extsb => sv as i8 as i32 as u32,
                    UnOp::Extsh => sv as i16 as i32 as u32,
                };
                c.gpr[a as usize] = v;
                if rc {
                    c.cr0(v);
                }
            }
            Op::Load { d, a, ea, size, sign, update, rev } => {
                let addr = self.ea(a, ea, update);
                let v = match size {
                    1 => mem.ld8(addr)? as u32,
                    2 => {
                        let h = mem.ld16(addr)?;
                        let h = if rev { h.swap_bytes() } else { h };
                        if sign { h as i16 as i32 as u32 } else { h as u32 }
                    }
                    _ => {
                        let x = mem.ld32(addr)?;
                        if rev { x.swap_bytes() } else { x }
                    }
                };
                let c = &mut self.cpu;
                c.gpr[d as usize] = v;
                if update {
                    c.gpr[a as usize] = addr;
                }
            }
            Op::Store { s, a, ea, size, update, rev } => {
                let addr = self.ea(a, ea, update);
                let v = self.cpu.r(s);
                match size {
                    1 => mem.st8(addr, v as u8)?,
                    2 => mem.st16(addr, if rev { (v as u16).swap_bytes() } else { v as u16 })?,
                    _ => mem.st32(addr, if rev { v.swap_bytes() } else { v })?,
                }
                if update {
                    self.cpu.gpr[a as usize] = addr;
                }
            }
            Op::LoadF { d, a, ea, single, update } => {
                let addr = self.ea(a, ea, update);
                let bits = if single {
                    let v = mem.ld32(addr)?;
                    if self.gekko { dol::from_single(v) } else { (f32::from_bits(v) as f64).to_bits() }
                } else {
                    mem.ld64(addr)?
                };
                self.cpu.fpr[d as usize] = bits;
                if update {
                    self.cpu.gpr[a as usize] = addr;
                }
            }
            Op::StoreF { s, a, ea, single, update } => {
                let addr = self.ea(a, ea, update);
                if single {
                    let v = if self.gekko { dol::to_single(self.cpu.fpr[s as usize]) } else { (self.cpu.f(s) as f32).to_bits() };
                    mem.st32(addr, v)?;
                } else {
                    mem.st64(addr, self.cpu.fpr[s as usize])?;
                }
                if update {
                    self.cpu.gpr[a as usize] = addr;
                }
            }
            Op::Stfiwx { s, a, b } => {
                let addr = self.ea(a, Ea::Idx(b), false);
                mem.st32(addr, self.cpu.fpr[s as usize] as u32)?;
            }
            Op::Lmw { d, a, disp } => {
                let addr = self.cpu.r0(a).wrapping_add(disp as u32);
                for i in d as u32..=31 {
                    self.cpu.gpr[i as usize] = mem.ld32(addr.wrapping_add(4 * (i - d as u32)))?;
                }
            }
            Op::Stmw { s, a, disp } => {
                let addr = self.cpu.r0(a).wrapping_add(disp as u32);
                for i in s as u32..=31 {
                    mem.st32(addr.wrapping_add(4 * (i - s as u32)), self.cpu.gpr[i as usize])?;
                }
            }
            Op::Dcbz { a, b } => {
                let addr = self.ea(a, Ea::Idx(b), false) & if self.gekko { !31 } else { !0 };
                for i in (0..32).step_by(4) {
                    mem.st32(addr.wrapping_add(i), 0)?;
                }
            }
            Op::Nop => {}
            Op::B { target, lk } => return self.branch(mem, br, target, lk),
            Op::Bc { bo, bi, target, lk } => {
                if self.cpu.cond(bo, bi) {
                    return self.branch(mem, br, target, lk);
                }
            }
            Op::Bclr { bo, bi, lk } | Op::Bcctr { bo, bi, lk } => {
                let is_ctr = matches!(op, Op::Bcctr { .. });
                if self.cpu.cond(bo, bi) {
                    let target = (if is_ctr { self.cpu.ctr } else { self.cpu.lr }) & if self.gekko { !3 } else { !0 };
                    if lk {
                        self.cpu.lr = self.cpu.pc;
                    }
                    if br.in_blob(target) {
                        self.cpu.pc = target;
                    } else if !is_ctr && !lk {
                        return Ok(Step::Return);
                    } else if lk {
                        br.call(&mut self.cpu, mem, target)?;
                    } else {
                        return self.tail(mem, br, target);
                    }
                }
            }
            Op::Mcrf { d, s } => {
                let f = c.cr_get_field(s);
                c.cr_set_field(d, f);
            }
            Op::CrLog { op, t, a, b } => {
                let (x, y) = (c.cr_bit(a), c.cr_bit(b));
                let r = match op {
                    CrOp::Nor => (x | y == 0) as u32,
                    CrOp::Andc => x & (y == 0) as u32,
                    CrOp::Xor => x ^ y,
                    CrOp::Nand => (x & y == 0) as u32,
                    CrOp::And => x & y,
                    CrOp::Eqv => (x ^ y == 0) as u32,
                    CrOp::Orc => x | (y == 0) as u32,
                    CrOp::Or => x | y,
                };
                c.cr_set_bit(t, r);
            }
            Op::Mfcr { d } => c.gpr[d as usize] = c.cr,
            Op::Mtcrf { mask, s } => c.cr = (c.cr & !mask) | (c.r(s) & mask),
            Op::Mfspr { d, spr } => {
                c.gpr[d as usize] = match spr {
                    Spr::Xer => c.xer,
                    Spr::Lr => c.lr,
                    Spr::Ctr => c.ctr,
                }
            }
            Op::Mtspr { spr, s } => {
                let v = c.r(s);
                match spr {
                    // SetXER keeps SO, OV, CA, the byte count and the compare byte
                    Spr::Xer => c.xer = if gekko { v & 0xE000_FF7F } else { v },
                    Spr::Lr => c.lr = v,
                    Spr::Ctr => c.ctr = v,
                }
            }
            Op::BadSpr { .. } => return Err(Fault::Panic(ip)),
            Op::FArith { op, d, a, b, c: fc, single } if gekko => {
                let (x, y, z) = (c.fpr[a as usize], c.fpr[b as usize], c.fpr[fc as usize]);
                let z = if single && matches!(op, FOp::Mul | FOp::Msub | FOp::Madd | FOp::Nmsub | FOp::Nmadd) { dol::force25(z) } else { z };
                let f = f64::from_bits;
                let r = match op {
                    FOp::Div => dol::ni2(x, y, f(x) / f(y)),
                    FOp::Sub => dol::ni2(x, y, f(x) - f(y)),
                    FOp::Add => dol::ni2(x, y, f(x) + f(y)),
                    FOp::Mul => dol::ni2(x, z, f(x) * f(z)),
                    FOp::Sqrt => f(y).sqrt().to_bits(),
                    // the estimates are stored as computed (Fill / SetPS0 of the double), never rounded to single
                    FOp::Res => melee_core::fp::fres(f(y)).to_bits(),
                    FOp::Rsqrte => melee_core::fp::frsqrte(f(y)).to_bits(),
                    FOp::Madd | FOp::Nmadd => dol::ni_fma(x, z, y, false),
                    FOp::Msub | FOp::Nmsub => dol::ni_fma(x, z, y, true),
                };
                let r = if single && !matches!(op, FOp::Res | FOp::Rsqrte) { ((f(r) as f32) as f64).to_bits() } else { r };
                // fnmadd / fnmsub negate the (rounded) result unless it is a NaN
                let r = if matches!(op, FOp::Nmadd | FOp::Nmsub) && !dol::is_nan(r) { r ^ dol::SIGN } else { r };
                c.fpr[d as usize] = r;
            }
            Op::FArith { op, d, a, b, c: fc, single } => {
                let (x, y, mut z) = (c.f(a), c.f(b), c.f(fc));
                // Gekko (Dolphin's Force25Bit): a single-precision multiply (fmuls, fmadds...) takes frC with 25
                // mantissa bits; a value that is already single is unchanged
                if gekko && single && matches!(op, FOp::Mul | FOp::Msub | FOp::Madd | FOp::Nmsub | FOp::Nmadd) {
                    z = melee_core::fp::force25(z);
                }
                let r = match op {
                    FOp::Div => x / y,
                    FOp::Sub => x - y,
                    FOp::Add => x + y,
                    FOp::Sqrt => y.sqrt(),
                    // the hardware estimates (table based, not correctly rounded), as the native port has them
                    FOp::Res if gekko => melee_core::fp::fres(y),
                    FOp::Rsqrte if gekko => melee_core::fp::frsqrte(y),
                    FOp::Res => 1.0 / y,
                    FOp::Mul => x * z,
                    FOp::Rsqrte => 1.0 / y.sqrt(),
                    FOp::Msub if gekko => x.mul_add(z, -y),
                    FOp::Madd if gekko => x.mul_add(z, y),
                    FOp::Nmsub if gekko => -x.mul_add(z, -y),
                    FOp::Nmadd if gekko => -x.mul_add(z, y),
                    FOp::Msub => (x * z) - y,
                    FOp::Madd => (x * z) + y,
                    FOp::Nmsub => -((x * z) - y),
                    FOp::Nmadd => -((x * z) + y),
                };
                c.set_f(d, if single { r as f32 as f64 } else { r });
            }
            Op::Fsel { d, a, b, c: fc } => {
                let x = c.f(a);
                // Dolphin: frA >= -0.0 ? frC : frB (a NaN selects frB)
                let v = if x >= 0.0 || (!gekko && x.is_nan()) { c.fpr[fc as usize] } else { c.fpr[b as usize] };
                c.fpr[d as usize] = v;
            }
            Op::FUnary { op, d, b } => {
                let y = c.f(b);
                match op {
                    FUn::Mr => c.fpr[d as usize] = c.fpr[b as usize],
                    FUn::Neg => c.set_f(d, -y),
                    // sign-bit operations (fabs of -0 / -NaN clears the sign)
                    FUn::Abs if gekko => c.fpr[d as usize] = c.fpr[b as usize] & !dol::SIGN,
                    FUn::Nabs if gekko => c.fpr[d as usize] = c.fpr[b as usize] | dol::SIGN,
                    FUn::Abs => c.set_f(d, if y < 0.0 { -y } else { y }),
                    FUn::Nabs => c.set_f(d, if y < 0.0 { y } else { -y }),
                    FUn::Rsp => c.set_f(d, y as f32 as f64),
                    FUn::Ctiwz if gekko => c.fpr[d as usize] = ctiw_gekko(y, false),
                    FUn::Ctiw if gekko => c.fpr[d as usize] = ctiw_gekko(y, true),
                    FUn::Ctiwz => c.fpr[d as usize] = ctiw(y),
                    FUn::Ctiw => c.fpr[d as usize] = ctiw(if y >= 0.0 { y + 0.5 } else { y - 0.5 }),
                }
            }
            Op::Fcmp { crf, a, b } => {
                let (x, y) = (c.f(a), c.f(b));
                // Gekko: the field is FPCC (FL FG FE FU); unordered sets FU, XER's SO is not involved
                let so = if gekko { (x.is_nan() || y.is_nan()) as u32 } else { c.xer >> 31 };
                let field = if x.is_nan() || y.is_nan() {
                    so
                } else if x > y {
                    4 | so
                } else if x < y {
                    8 | so
                } else {
                    2 | so
                };
                c.cr_set_field(crf, field);
            }
            Op::Bad => return Err(Fault::BadOpcode(ip, w)),
        }
        Ok(Step::Continue)
    }
}
