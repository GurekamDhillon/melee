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
    0xFFF8_0000_0000_0000 | (iv as u32 as u64)
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
                let b = c.r0(a);
                c.set_ca(imm >= b);
                c.gpr[d as usize] = imm.wrapping_sub(b);
            }
            Op::AddIc { d, a, imm, rc } => {
                let b = c.r0(a);
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
                let bits = if single { (f32::from_bits(mem.ld32(addr)?) as f64).to_bits() } else { mem.ld64(addr)? };
                self.cpu.fpr[d as usize] = bits;
                if update {
                    self.cpu.gpr[a as usize] = addr;
                }
            }
            Op::StoreF { s, a, ea, single, update } => {
                let addr = self.ea(a, ea, update);
                if single {
                    mem.st32(addr, (self.cpu.f(s) as f32).to_bits())?;
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
                let addr = self.ea(a, Ea::Idx(b), false);
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
                    let target = if is_ctr { self.cpu.ctr } else { self.cpu.lr };
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
                    Spr::Xer => c.xer = v,
                    Spr::Lr => c.lr = v,
                    Spr::Ctr => c.ctr = v,
                }
            }
            Op::BadSpr { .. } => return Err(Fault::Panic(ip)),
            Op::FArith { op, d, a, b, c: fc, single } => {
                let (x, y, z) = (c.f(a), c.f(b), c.f(fc));
                let r = match op {
                    FOp::Div => x / y,
                    FOp::Sub => x - y,
                    FOp::Add => x + y,
                    FOp::Sqrt => y.sqrt(),
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
                let v = if x >= 0.0 || x.is_nan() { c.fpr[fc as usize] } else { c.fpr[b as usize] };
                c.fpr[d as usize] = v;
            }
            Op::FUnary { op, d, b } => {
                let y = c.f(b);
                match op {
                    FUn::Mr => c.fpr[d as usize] = c.fpr[b as usize],
                    FUn::Neg => c.set_f(d, -y),
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
                let so = c.xer >> 31;
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
